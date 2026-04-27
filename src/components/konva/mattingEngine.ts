import Konva from "konva";
import type { KonvaEngineApi } from "./types";

const MIN_ZOOM = 0.001;
const MAX_ZOOM = 2;
const CHECKER_CELL = 10;

function createCheckerCanvas(cellPx = 10): HTMLCanvasElement {
  const s = cellPx * 2;
  const c = document.createElement("canvas");
  c.width = s;
  c.height = s;
  const ctx = c.getContext("2d");
  if (!ctx) return c;
  ctx.fillStyle = "#f0f0f0";
  ctx.fillRect(0, 0, s, s);
  ctx.fillStyle = "#d4d4d4";
  ctx.fillRect(0, 0, cellPx, cellPx);
  ctx.fillRect(cellPx, cellPx, cellPx, cellPx);
  return c;
}

/** Composite source with mask using destination-in (Porter-Duff).
 *  Mask is grayscale: white=keep, black=remove. */
function compositeWithMask(
  source: HTMLImageElement,
  maskImg: HTMLImageElement,
): HTMLCanvasElement {
  const w = source.naturalWidth || source.width;
  const h = source.naturalHeight || source.height;

  const out = document.createElement("canvas");
  out.width = w;
  out.height = h;
  const ctx = out.getContext("2d");
  if (!ctx) return out;

  ctx.drawImage(source, 0, 0, w, h);

  const maskCanvas = document.createElement("canvas");
  maskCanvas.width = w;
  maskCanvas.height = h;
  const mctx = maskCanvas.getContext("2d");
  if (!mctx) return out;
  mctx.drawImage(maskImg, 0, 0, w, h);

  const maskData = mctx.getImageData(0, 0, w, h);
  const lumCanvas = document.createElement("canvas");
  lumCanvas.width = w;
  lumCanvas.height = h;
  const lctx = lumCanvas.getContext("2d");
  if (!lctx) return out;

  const lumData = lctx.createImageData(w, h);
  for (let i = 0; i < lumData.data.length; i += 4) {
    const lum = (maskData.data[i]! + maskData.data[i + 1]! + maskData.data[i + 2]!) / (3 * 255);
    lumData.data[i] = 255;
    lumData.data[i + 1] = 255;
    lumData.data[i + 2] = 255;
    lumData.data[i + 3] = Math.round(lum * 255);
  }
  lctx.putImageData(lumData, 0, 0);

  ctx.save();
  ctx.globalCompositeOperation = "destination-in";
  ctx.drawImage(lumCanvas, 0, 0, w, h);
  ctx.restore();

  return out;
}

function clamp(n: number, a: number, b: number): number {
  return Math.min(b, Math.max(a, n));
}

function parseAlpha(input: string): number {
  const t = input.trim();
  const rgba = t.match(
    /^rgba\s*\(\s*[\d.]+\s*,\s*[\d.]+\s*,\s*[\d.]+\s*,\s*([\d.]+)\s*\)/i,
  );
  if (rgba) {
    const a = Number(rgba[1]);
    return Number.isFinite(a) ? clamp(a, 0, 1) : 1;
  }
  if (/^rgb\s*\(/i.test(t) && !/^rgba\s*\(/i.test(t)) return 1;
  let h = t.replace(/^#/, "");
  if (h.length === 3) h = h.split("").map((ch) => ch + ch).join("");
  if (h.length === 8) {
    const aByte = parseInt(h.slice(6, 8), 16);
    return Number.isFinite(aByte) ? clamp(aByte / 255, 0, 1) : 1;
  }
  if (h.length === 6) return 1;
  return 1;
}

export interface MattingEngineOptions {
  onViewportChange?: (scale: number, fit: number) => void;
  onDocumentSizeChange?: (size: { width: number; height: number }) => void;
  compareHtml?: HTMLImageElement | null;
}

export function createMattingEngine(
  container: HTMLDivElement,
  stageWidth: number,
  stageHeight: number,
  processedHtml: HTMLImageElement,
  options: MattingEngineOptions = {},
): { api: KonvaEngineApi; destroy: () => void } {
  const { onViewportChange, onDocumentSizeChange, compareHtml } = options;
  const notifyDocSize = () => onDocumentSizeChange?.({ width: docW, height: docH });

  const nw = processedHtml.naturalWidth || processedHtml.width;
  const nh = processedHtml.naturalHeight || processedHtml.height;

  let docW = nw;
  let docH = nh;

  const stage = new Konva.Stage({ container, width: stageWidth, height: stageHeight });
  const layer = new Konva.Layer();

  const world = new Konva.Group({ name: "world" });
  const checkerPattern = createCheckerCanvas(CHECKER_CELL);

  const transparencyGrid = new Konva.Rect({
    name: "transparencyGrid",
    x: 0, y: 0,
    width: docW, height: docH,
    fillPatternImage: checkerPattern as unknown as HTMLImageElement,
    fillPatternRepeat: "repeat",
    listening: false,
  });

  const artboard = new Konva.Group({
    name: "artboard",
    clip: { x: 0, y: 0, width: docW, height: docH },
  });

  const bgUnder = new Konva.Group({ listening: false, name: "bgUnder" });
  const mainGroup = new Konva.Group({ name: "mainGroup" });

  let bgMode: "none" | "solidTranslucent" | "solidOpaque" | "gradient" | "image" | "checkerboard" = "none";
  let bgUnderOpacity01 = 1;
  let exportSolidColor = "#ffffff";
  let exportGradient: { x1: number; y1: number; x2: number; y2: number; stops: (number | string)[] } | null = null;
  let exportBgImage: HTMLImageElement | null = null;
  const applyBgUnderOpacity = () => bgUnder.opacity(bgUnderOpacity01);

  // Initial composited image (no mask, full image). Mask will be applied via updateMask().
  const composedCanvas = document.createElement("canvas");
  composedCanvas.width = nw;
  composedCanvas.height = nh;
  const cctx = composedCanvas.getContext("2d");
  if (cctx) cctx.drawImage(processedHtml, 0, 0, nw, nh);

  const kImage = new Konva.Image({
    image: composedCanvas,
    x: 0, y: 0,
    width: nw, height: nh,
  });
  mainGroup.add(kImage);

  let compareImageNode: Konva.Image | null = null;
  if (compareHtml) {
    compareImageNode = new Konva.Image({
      image: compareHtml,
      x: 0, y: 0,
      width: compareHtml.naturalWidth || compareHtml.width,
      height: compareHtml.naturalHeight || compareHtml.height,
      visible: false,
    });
    mainGroup.add(compareImageNode);
  }

  artboard.add(bgUnder);
  artboard.add(mainGroup);
  world.add(artboard);

  layer.add(transparencyGrid);
  layer.add(world);
  stage.add(layer);

  let worldScale = 1;
  let worldX = 0;
  let worldY = 0;
  let containScale = 1;
  let mainGroupX = 0;
  let mainGroupY = 0;
  let compareActive = false;

  const fitDoc = () =>
    Math.min(stage.width() / Math.max(1, docW), stage.height() / Math.max(1, docH));

  const emitViewport = () => onViewportChange?.(worldScale, fitDoc());

  const syncTransparencyGridPos = () => {
    if (!transparencyGrid.visible()) return;
    transparencyGrid.setAttrs({
      x: worldX, y: worldY,
      width: Math.max(1, docW * worldScale),
      height: Math.max(1, docH * worldScale),
    });
  };

  const syncTgVisibility = () => {
    const through = bgUnderOpacity01 < 1 - 1e-5 && (bgMode === "solidOpaque" || bgMode === "image");
    transparencyGrid.visible(bgMode === "none" || bgMode === "solidTranslucent" || bgMode === "checkerboard" || through);
    if (transparencyGrid.visible()) syncTransparencyGridPos();
  };

  const setBackgroundCheckerboard = () => {
    bgMode = "checkerboard";
    exportGradient = null;
    exportBgImage = null;
    clearBgNodes();
    resetBgOpacity();
    syncTgVisibility();
    layer.batchDraw();
  };

  const applyWorldTransform = () => {
    world.position({ x: worldX, y: worldY });
    world.scale({ x: worldScale, y: worldScale });
    syncTransparencyGridPos();
    layer.batchDraw();
    emitViewport();
  };

  const centerWorld = () => {
    worldX = (stage.width() - docW * worldScale) / 2;
    worldY = (stage.height() - docH * worldScale) / 2;
    applyWorldTransform();
  };

  const layoutContainImage = () => {
    containScale = Math.min(docW / Math.max(1, nw), docH / Math.max(1, nh));
    const iw = nw * containScale;
    const ih = nh * containScale;
    mainGroupX = (docW - iw) / 2;
    mainGroupY = (docH - ih) / 2;
    mainGroup.position({ x: mainGroupX, y: mainGroupY });
    mainGroup.scale({ x: containScale, y: containScale });

    if (!compareActive) {
      kImage.visible(true);
      if (compareImageNode) compareImageNode.visible(false);
    } else if (compareImageNode) {
      kImage.visible(false);
      compareImageNode.visible(true);
      const ow = compareImageNode.width();
      const oh = compareImageNode.height();
      const cs = Math.min(iw / Math.max(1, ow), ih / Math.max(1, oh));
      compareImageNode.scale({ x: cs, y: cs });
      compareImageNode.position({
        x: (iw - ow * cs) / 2,
        y: (ih - oh * cs) / 2,
      });
    }
    layer.batchDraw();
  };

  const zoomWorldToPoint = (sx: number, sy: number, ns: number) => {
    const cs = clamp(ns, MIN_ZOOM, MAX_ZOOM);
    const lx = (sx - worldX) / worldScale;
    const ly = (sy - worldY) / worldScale;
    worldScale = cs;
    worldX = sx - lx * worldScale;
    worldY = sy - ly * worldScale;
    applyWorldTransform();
  };

  // Pan
  let viewportPan: { lx: number; ly: number; pid: number } | null = null;

  const clearPan = () => {
    if (viewportPan) {
      try { stage.container().releasePointerCapture(viewportPan.pid); } catch { /* ok */ }
      viewportPan = null;
    }
  };

  const onPtrDown = (e: PointerEvent) => {
    if (e.pointerType === "mouse" && e.button !== 1) return;
    if (e.pointerType === "touch" && !e.isPrimary) return;
    viewportPan = { lx: e.clientX, ly: e.clientY, pid: e.pointerId };
    try { stage.container().setPointerCapture(e.pointerId); } catch { /* ok */ }
    e.preventDefault();
  };

  const onPtrMove = (e: PointerEvent) => {
    if (!viewportPan || viewportPan.pid !== e.pointerId) return;
    const dx = e.clientX - viewportPan.lx;
    const dy = e.clientY - viewportPan.ly;
    viewportPan.lx = e.clientX;
    viewportPan.ly = e.clientY;
    worldX += dx;
    worldY += dy;
    applyWorldTransform();
    e.preventDefault();
  };

  const onPtrUp = (e: PointerEvent) => {
    if (!viewportPan || viewportPan.pid !== e.pointerId) return;
    clearPan();
  };

  const onWheel = (e: Konva.KonvaEventObject<WheelEvent>) => {
    e.evt.preventDefault();
    const p = stage.getPointerPosition();
    if (!p) return;
    zoomWorldToPoint(p.x, p.y, worldScale * (1 + (e.evt.deltaY > 0 ? -1 : 1) * 0.08));
  };

  let pinchD = 0;
  let pinchS = 1;
  let pinchMx = 0;
  let pinchMy = 0;

  const touchDist = (tl: TouchList) => {
    if (tl.length < 2) return 0;
    return Math.hypot(tl[0]!.clientX - tl[1]!.clientX, tl[0]!.clientY - tl[1]!.clientY);
  };

  const onTS = (e: TouchEvent) => {
    if (e.touches.length === 2) {
      pinchD = touchDist(e.touches);
      pinchS = worldScale;
      pinchMx = (e.touches[0]!.clientX + e.touches[1]!.clientX) / 2;
      pinchMy = (e.touches[0]!.clientY + e.touches[1]!.clientY) / 2;
    }
  };

  const onTM = (e: TouchEvent) => {
    if (e.touches.length !== 2 || pinchD <= 0) return;
    e.preventDefault();
    const d = touchDist(e.touches);
    if (d <= 0) return;
    zoomWorldToPoint(pinchMx, pinchMy, pinchS * (d / pinchD));
  };

  const onTE = (e: TouchEvent) => {
    if (e.touches.length < 2) pinchD = 0;
  };

  const cel = stage.container();
  cel.addEventListener("pointerdown", onPtrDown, true);
  cel.addEventListener("pointermove", onPtrMove, true);
  cel.addEventListener("pointerup", onPtrUp, true);
  cel.addEventListener("pointercancel", onPtrUp, true);
  cel.addEventListener("touchstart", onTS, { passive: false });
  cel.addEventListener("touchmove", onTM, { passive: false });
  cel.addEventListener("touchend", onTE);
  cel.addEventListener("touchcancel", onTE);
  stage.on("wheel", onWheel);

  const clearBgNodes = () => bgUnder.destroyChildren();
  const resetBgOpacity = () => { bgUnderOpacity01 = 1; applyBgUnderOpacity(); };

  const api: KonvaEngineApi & { updateMask: (maskImg: HTMLImageElement) => void } = {
    stage,
    resizeStage: (w, h) => {
      stage.width(w);
      stage.height(h);
      worldScale = clamp(fitDoc(), MIN_ZOOM, MAX_ZOOM);
      centerWorld();
    },
    getNaturalSize: () => ({ width: nw, height: nh }),
    getDocumentSize: () => ({ width: docW, height: docH }),
    getViewportScale: () => worldScale,
    getViewportFitScale: () => fitDoc(),
    setAbsoluteViewportZoom: (z) => {
      if (!(z > 0)) return;
      worldScale = clamp(z, MIN_ZOOM, MAX_ZOOM);
      centerWorld();
    },
    zoomViewportToPoint: (sx, sy, z) => { if (z > 0) zoomWorldToPoint(sx, sy, z); },
    panViewportBy: (dx, dy) => { worldX += dx; worldY += dy; applyWorldTransform(); },
    setCompareOriginal: (show) => {
      if (show && !compareImageNode) return;
      compareActive = show && !!compareImageNode;
      layoutContainImage();
    },
    clearBackground: () => {
      bgMode = "none";
      clearBgNodes();
      resetBgOpacity();
      exportSolidColor = "#ffffff";
      exportGradient = null;
      exportBgImage = null;
      syncTgVisibility();
      layer.batchDraw();
    },
    setBackgroundCheckerboard,
    setBackgroundOpacity: (pct) => {
      bgUnderOpacity01 = clamp(pct / 100, 0, 1);
      applyBgUnderOpacity();
      syncTgVisibility();
      layer.batchDraw();
    },
    setBackgroundSolidColor: (hex) => {
      const a = parseAlpha(hex);
      bgMode = a < 1 - 1e-5 ? "solidTranslucent" : "solidOpaque";
      exportSolidColor = hex;
      exportGradient = null;
      exportBgImage = null;
      clearBgNodes();
      bgUnder.add(new Konva.Rect({
        x: 0, y: 0, width: docW, height: docH,
        fill: hex, listening: false,
      }));
      syncTgVisibility();
      layer.batchDraw();
    },
    setBackgroundImageFromUrl: (url) =>
      new Promise<void>((resolve, reject) => {
        const im = new window.Image();
        im.crossOrigin = "anonymous";
        im.onload = () => {
          bgMode = "image";
          exportBgImage = im;
          exportGradient = null;
          clearBgNodes();
          const node = new Konva.Image({ image: im, listening: false });
          const iw = im.naturalWidth || im.width;
          const ih = im.naturalHeight || im.height;
          const sc = Math.max(docW / iw, docH / ih);
          node.setAttrs({
            image: im,
            x: docW / 2, y: docH / 2,
            offsetX: iw / 2, offsetY: ih / 2,
            width: iw, height: ih,
            scaleX: sc, scaleY: sc,
            listening: false,
          });
          bgUnder.add(node);
          syncTgVisibility();
          layer.batchDraw();
          resolve();
        };
        im.onerror = () => reject(new Error("Background image load failed"));
        im.src = url;
      }),
    setBackgroundLinearGradientPixels: (opts) => {
      if (opts.colorStops.length < 2) return;
      bgMode = "gradient";
      resetBgOpacity();
      const stops: (number | string)[] = [];
      opts.colorStops.forEach((cs) => { stops.push(cs.offset, cs.color); });
      exportGradient = { x1: opts.x1, y1: opts.y1, x2: opts.x2, y2: opts.y2, stops };
      exportBgImage = null;
      clearBgNodes();
      bgUnder.add(new Konva.Rect({
        x: 0, y: 0, width: docW, height: docH,
        fillLinearGradientStartPoint: { x: opts.x1, y: opts.y1 },
        fillLinearGradientEndPoint: { x: opts.x2, y: opts.y2 },
        fillLinearGradientColorStops: stops,
        listening: false,
      }));
      syncTgVisibility();
      layer.batchDraw();
    },
    getExportPngDataUrl: (pr = 1, mt = "image/png", q) => {
      const c = document.createElement("canvas");
      c.width = docW * pr;
      c.height = docH * pr;
      const ctx = c.getContext("2d");
      if (!ctx) return "";

      ctx.scale(pr, pr);

      const alpha = bgUnderOpacity01;

      // Draw checkerboard pattern
      if (bgMode === "checkerboard") {
        const patternCanvas = createCheckerCanvas(CHECKER_CELL);
        const pattern = ctx.createPattern(patternCanvas, "repeat");
        if (pattern) {
          ctx.fillStyle = pattern;
          ctx.fillRect(0, 0, docW, docH);
        }
      }

      // Draw solid color background
      if ((bgMode === "solidOpaque" || bgMode === "solidTranslucent")) {
        ctx.save();
        if (alpha < 1 - 1e-5) ctx.globalAlpha = alpha;
        ctx.fillStyle = exportSolidColor;
        ctx.fillRect(0, 0, docW, docH);
        ctx.restore();
      }

      // Draw gradient background
      if (bgMode === "gradient" && exportGradient) {
        ctx.save();
        const g = exportGradient;
        const grad = ctx.createLinearGradient(g.x1, g.y1, g.x2, g.y2);
        for (let i = 0; i < g.stops.length; i += 2) {
          grad.addColorStop(g.stops[i] as number, g.stops[i + 1] as string);
        }
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, docW, docH);
        ctx.restore();
      }

      // Draw image background
      if (bgMode === "image" && exportBgImage) {
        ctx.save();
        if (alpha < 1 - 1e-5) ctx.globalAlpha = alpha;
        const img = exportBgImage;
        const iw = img.naturalWidth || img.width;
        const ih = img.naturalHeight || img.height;
        const sc = Math.max(docW / iw, docH / ih);
        const dw = iw * sc;
        const dh = ih * sc;
        ctx.drawImage(img, (docW - dw) / 2, (docH - dh) / 2, dw, dh);
        ctx.restore();
      }

      // Draw composited subject image (centered, retains transparency)
      const s = containScale;
      const iw = nw * s;
      const ih = nh * s;
      ctx.drawImage(composedCanvas, (docW - iw) / 2, (docH - ih) / 2, iw, ih);

      // JPEG: flatten any remaining alpha onto white
      if (mt === "image/jpeg") {
        const id = ctx.getImageData(0, 0, docW, docH);
        const d = id.data;
        for (let i = 3; i < d.length; i += 4) {
          if (d[i] < 255) {
            const a = d[i] / 255;
            d[i - 3] = d[i - 3] * a + 255 * (1 - a);
            d[i - 2] = d[i - 2] * a + 255 * (1 - a);
            d[i - 1] = d[i - 1] * a + 255 * (1 - a);
            d[i] = 255;
          }
        }
        ctx.putImageData(id, 0, 0);
      }

      return c.toDataURL(mt, q);
    },
    destroy: () => {
      clearPan();
      cel.removeEventListener("pointerdown", onPtrDown, true);
      cel.removeEventListener("pointermove", onPtrMove, true);
      cel.removeEventListener("pointerup", onPtrUp, true);
      cel.removeEventListener("pointercancel", onPtrUp, true);
      cel.removeEventListener("touchstart", onTS);
      cel.removeEventListener("touchmove", onTM);
      cel.removeEventListener("touchend", onTE);
      cel.removeEventListener("touchcancel", onTE);
      stage.off("wheel", onWheel);
      stage.destroy();
    },
    onViewportChange,
    onDocumentSizeChange,

    /** Apply mask to the composited image. Call after mask image loads. */
    updateMask: (maskImg: HTMLImageElement) => {
      const canvas = compositeWithMask(processedHtml, maskImg);
      composedCanvas.width = canvas.width;
      composedCanvas.height = canvas.height;
      const cc = composedCanvas.getContext("2d");
      if (cc) {
        cc.clearRect(0, 0, composedCanvas.width, composedCanvas.height);
        cc.drawImage(canvas, 0, 0);
      }
      kImage.image(composedCanvas);
      layer.batchDraw();
    },
  };

  // Initial layout
  layoutContainImage();
  syncTgVisibility();
  const fit = fitDoc();
  worldScale = fit;
  centerWorld();
  notifyDocSize();

  const destroy = () => api.destroy();
  return { api, destroy };
}
