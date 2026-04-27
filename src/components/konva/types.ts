import type Konva from "konva";

export interface KonvaEngineApi {
  stage: Konva.Stage;
  resizeStage: (width: number, height: number) => void;
  getNaturalSize: () => { width: number; height: number };
  getDocumentSize: () => { width: number; height: number };
  getViewportScale: () => number;
  getViewportFitScale: () => number;
  setAbsoluteViewportZoom: (zoom: number) => void;
  zoomViewportToPoint: (stageX: number, stageY: number, zoom: number) => void;
  panViewportBy: (dx: number, dy: number) => void;
  setCompareOriginal: (show: boolean) => void;
  clearBackground: () => void;
  setBackgroundCheckerboard: () => void;
  setBackgroundOpacity: (opacityPercent: number) => void;
  setBackgroundSolidColor: (hex: string) => void;
  setBackgroundImageFromUrl: (url: string) => Promise<void>;
  setBackgroundLinearGradientPixels: (opts: {
    x1: number; y1: number; x2: number; y2: number;
    colorStops: { offset: number; color: string }[];
  }) => void;
  getExportPngDataUrl: (pixelRatio?: number, mimeType?: string, quality?: number) => string;
  destroy: () => void;
  updateMask: (maskImg: HTMLImageElement) => void;
  onViewportChange?: (scale: number, fit: number) => void;
  onDocumentSizeChange?: (size: { width: number; height: number }) => void;
}
