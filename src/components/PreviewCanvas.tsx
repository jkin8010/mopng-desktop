import { useEffect, useRef, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ZoomIn, ZoomOut, RotateCcw, ImageOff, Loader2 } from "lucide-react";
import { useStore } from "@/store";
import { cn } from "@/lib/utils";
import { createMattingEngine } from "./konva/mattingEngine";
import type { KonvaEngineApi } from "./konva/types";
import type { MattingTask } from "@/types";

interface PreviewCanvasProps {
  task: MattingTask | null;
}

export function PreviewCanvas({ task }: PreviewCanvasProps) {
  const currentSettings = useStore((s) => s.currentSettings);
  const modelSwitching = useStore((s) => s.modelSwitching);
  const activeModelId = useStore((s) => s.activeModelId);
  const availableModels = useStore((s) => s.availableModels);
  const switchingModelName = availableModels.find((m) => m.id === activeModelId)?.name || activeModelId;
  const containerRef = useRef<HTMLDivElement>(null);
  const konvaHostRef = useRef<HTMLDivElement>(null);
  const engineRef = useRef<{ api: KonvaEngineApi; destroy: () => void } | null>(null);
  const [engineReady, setEngineReady] = useState(false);
  const [zoomPercent, setZoomPercent] = useState(100);
  const [fitPercent, setFitPercent] = useState(100);
  const [showOriginal, setShowOriginal] = useState(false);
  const hasMask = task?.result?.maskDataUrl != null;

  // Sync background settings to engine
  const applyBackground = useCallback(
    (api: KonvaEngineApi) => {
      const s = useStore.getState().currentSettings;
      switch (s.bgType) {
        case "transparent":
          api.clearBackground();
          break;
        case "white":
          api.setBackgroundSolidColor("#ffffff");
          break;
        case "color":
          api.setBackgroundSolidColor(s.bgColor || "#ffffff");
          if (s.bgOpacity !== undefined) {
            api.setBackgroundOpacity(s.bgOpacity);
          }
          break;
        case "checkerboard":
          api.setBackgroundCheckerboard();
          break;
        case "image":
          if (s.bgImageUrl) {
            api.setBackgroundImageFromUrl(s.bgImageUrl);
            if (s.bgOpacity !== undefined) {
              api.setBackgroundOpacity(s.bgOpacity);
            }
          }
          break;
        case "gradient":
          if (s.bgGradient && s.bgGradient.colorStops.length >= 2) {
            const doc = api.getDocumentSize();
            api.setBackgroundLinearGradientPixels({
              x1: (s.bgGradient.x1 ?? 0) * doc.width,
              y1: (s.bgGradient.y1 ?? 0) * doc.height,
              x2: (s.bgGradient.x2 ?? 0) * doc.width,
              y2: (s.bgGradient.y2 ?? 0) * doc.height,
              colorStops: s.bgGradient.colorStops,
            });
            if (s.bgOpacity !== undefined) {
              api.setBackgroundOpacity(s.bgOpacity);
            }
          }
          break;
      }
    },
    [],
  );

  // Create/destroy engine when task changes
  useEffect(() => {
    const host = konvaHostRef.current;
    if (!host || !task) return;

    // Clean up previous engine
    if (engineRef.current) {
      engineRef.current.destroy();
      engineRef.current = null;
    }
    setEngineReady(false);

    const hasResult = task.result?.previewPath != null;
    const mainPath = hasResult ? task.result!.previewPath! : task.filePath;
    const comparePath: string | null = hasResult ? task.filePath : null;

    let disposed = false;

    const initEngine = async () => {
      try {
        // Load images as data URLs (same-origin) to avoid canvas tainting
        const [mainDataUrl, compareDataUrl] = await Promise.all([
          invoke<string>("read_file_as_data_url", { path: mainPath }),
          comparePath
            ? invoke<string>("read_file_as_data_url", { path: comparePath })
            : Promise.resolve(null),
        ]);

        if (disposed) return;

        const mainImg = new window.Image();
        const compareImg = compareDataUrl ? new window.Image() : null;

        let loadedCount = 0;
        const neededCount = compareImg ? 2 : 1;

        const tryInit = () => {
          loadedCount++;
          if (loadedCount < neededCount || disposed) return;

          const rect = host.getBoundingClientRect();
          const w = rect.width || 800;
          const h = rect.height || 600;

          const s = useStore.getState().currentSettings;
          const docSize =
            s.targetWidth != null && s.targetHeight != null
              ? { width: s.targetWidth, height: s.targetHeight }
              : undefined;

          const engine = createMattingEngine(host, w, h, mainImg, {
            compareHtml: compareImg,
            docSize,
            maintainAspectRatio: s.maintainAspectRatio,
            onViewportChange: (scale, fit) => {
              setZoomPercent(Math.round(scale * 100));
              setFitPercent(Math.round(fit * 100));
            },
            onDocumentSizeChange: () => {
              const api = engineRef.current?.api;
              if (api) applyBackground(api);
            },
          });

          engineRef.current = engine;
          setEngineReady(true);

          useStore.getState().setKonvaExportFn((mimeType, quality) =>
            engine.api.getExportPngDataUrl(1, mimeType, quality),
          );

          applyBackground(engine.api);

          if (task.result?.maskDataUrl) {
            const maskImg = new window.Image();
            maskImg.onload = () => engine.api.updateMask(maskImg);
            maskImg.src = task.result.maskDataUrl;
          }
        };

        mainImg.onload = tryInit;
        mainImg.onerror = tryInit;
        mainImg.src = mainDataUrl;

        if (compareImg && compareDataUrl) {
          compareImg.onload = tryInit;
          compareImg.onerror = tryInit;
          compareImg.src = compareDataUrl;
        }
      } catch (e) {
        if (!disposed) {
          console.error("[PreviewCanvas] Failed to load images:", e);
        }
      }
    };

    initEngine();

    return () => {
      disposed = true;
      useStore.getState().setKonvaExportFn(null);
      if (engineRef.current) {
        engineRef.current.destroy();
        engineRef.current = null;
      }
      setEngineReady(false);
    };
  }, [task?.id, task?.result?.previewPath, task?.result?.maskDataUrl]);

  // Sync background settings reactively
  useEffect(() => {
    const api = engineRef.current?.api;
    if (!api) return;
    applyBackground(api);
  }, [
    currentSettings.bgType,
    currentSettings.bgColor,
    currentSettings.bgOpacity,
    currentSettings.bgImageUrl,
    currentSettings.bgGradient,
  ]);

  // Sync document size to engine when size template changes
  useEffect(() => {
    const api = engineRef.current?.api;
    if (!api) return;
    const tw = currentSettings.targetWidth;
    const th = currentSettings.targetHeight;
    if (tw != null && th != null) {
      api.setDocumentSize(tw, th);
    } else {
      const natural = api.getNaturalSize();
      api.setDocumentSize(natural.width, natural.height);
    }
  }, [currentSettings.targetWidth, currentSettings.targetHeight]);

  // Sync maintainAspectRatio to engine
  useEffect(() => {
    const api = engineRef.current?.api;
    if (!api) return;
    api.setMaintainAspectRatio(currentSettings.maintainAspectRatio);
  }, [currentSettings.maintainAspectRatio]);

  // ResizeObserver for responsive stage sizing
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const ro = new ResizeObserver((entries) => {
      const api = engineRef.current?.api;
      if (!api) return;
      const rect = entries[0]?.contentRect;
      if (!rect || rect.width <= 0 || rect.height <= 0) return;
      api.resizeStage(rect.width, rect.height);
    });

    ro.observe(container);
    return () => ro.disconnect();
  }, [engineReady]);

  // Zoom controls
  const zoomIn = () => {
    const api = engineRef.current?.api;
    if (!api) return;
    const cur = api.getViewportScale();
    api.setAbsoluteViewportZoom(Math.min(cur * 1.25, 2));
  };

  const zoomOut = () => {
    const api = engineRef.current?.api;
    if (!api) return;
    const cur = api.getViewportScale();
    api.setAbsoluteViewportZoom(Math.max(cur * 0.8, 0.001));
  };

  const zoomReset = () => {
    const api = engineRef.current?.api;
    if (!api) return;
    api.setAbsoluteViewportZoom(api.getViewportFitScale());
  };

  // Compare mode
  const startCompare = () => {
    if (!hasMask) return;
    setShowOriginal(true);
    engineRef.current?.api.setCompareOriginal(true);
  };

  const endCompare = () => {
    setShowOriginal(false);
    engineRef.current?.api.setCompareOriginal(false);
  };

  if (!task) {
    return (
      <div className="flex-1 flex items-center justify-center bg-muted/30">
        <div className="text-center text-muted-foreground">
          <ImageOff className="w-16 h-16 mx-auto mb-4 opacity-50" />
          <p className="text-lg font-medium">暂无图片</p>
          <p className="text-sm mt-1">拖拽图片到此处或点击左侧添加</p>
        </div>
      </div>
    );
  }

  return (
    <div ref={containerRef} className="flex-1 relative overflow-hidden bg-muted/30">
      {/* Konva host div */}
      <div ref={konvaHostRef} className="absolute inset-0" />

      {/* Processing overlay */}
      {task.status === "processing" && (
        <div className="absolute inset-0 flex items-center justify-center bg-background/50 z-10">
          <div className="flex flex-col items-center gap-3">
            <Loader2 className="w-12 h-12 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground">AI 抠图中...</p>
          </div>
        </div>
      )}

      {/* Error overlay */}
      {task.status === "error" && (
        <div className="absolute top-4 left-1/2 -translate-x-1/2 z-10 px-4 py-2 rounded-full bg-destructive text-destructive-foreground text-sm shadow-lg">
          错误: {task.error}
        </div>
      )}

      {/* Model switch loading overlay */}
      {modelSwitching && (
        <div className="absolute inset-0 z-20 flex items-center justify-center bg-background/85 backdrop-blur-sm transition-opacity duration-150">
          <div className="max-w-xs w-full mx-4 p-6 rounded-lg bg-card border border-border shadow-lg">
            <div className="flex flex-col items-center gap-4 text-center">
              <Loader2 className="w-10 h-10 animate-spin text-primary" />
              <div>
                <p className="text-sm font-semibold">正在切换至 {switchingModelName}</p>
                <p className="text-xs text-muted-foreground mt-1">正在加载模型...</p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Toolbar */}
      <div className="absolute bottom-4 left-1/2 -translate-x-1/2 z-10 flex items-center gap-1 p-1.5 rounded-lg bg-background/80 backdrop-blur-sm border border-border shadow-lg">
        <button
          onClick={zoomOut}
          className="p-2 rounded-md hover:bg-muted transition-colors"
          title="缩小"
        >
          <ZoomOut className="w-4 h-4" />
        </button>
        <span className="text-xs font-mono w-14 text-center">{zoomPercent}%</span>
        <button
          onClick={zoomIn}
          className="p-2 rounded-md hover:bg-muted transition-colors"
          title="放大"
        >
          <ZoomIn className="w-4 h-4" />
        </button>
        <div className="w-px h-5 bg-border mx-1" />
        <button
          onClick={zoomReset}
          className="p-2 rounded-md hover:bg-muted transition-colors"
          title="重置视图"
        >
          <RotateCcw className="w-4 h-4" />
        </button>
        {hasMask && (
          <>
            <div className="w-px h-5 bg-border mx-1" />
            <button
              onMouseDown={startCompare}
              onMouseUp={endCompare}
              onMouseLeave={endCompare}
              onTouchStart={startCompare}
              onTouchEnd={endCompare}
              className={cn(
                "px-3 py-1.5 text-xs rounded-md hover:bg-muted transition-colors",
                showOriginal && "bg-primary/20",
              )}
              title="按住查看原图"
            >
              按住对比
            </button>
          </>
        )}
      </div>

      {/* Document size indicator */}
      {task.result && (
        <div className="absolute top-4 right-4 z-10 text-xs text-muted-foreground bg-background/60 backdrop-blur-sm px-2 py-1 rounded">
          {currentSettings.targetWidth != null && currentSettings.targetHeight != null
            ? `${currentSettings.targetWidth} x ${currentSettings.targetHeight}`
            : `${task.result.width} x ${task.result.height}`}
        </div>
      )}
    </div>
  );
}
