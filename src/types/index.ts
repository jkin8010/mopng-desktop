export type MattingMode = "foreground" | "background";

export type OutputFormat = "png" | "jpg" | "webp";

export type BgType = "transparent" | "white" | "color" | "checkerboard" | "image" | "gradient";

export type GradientType = "linear" | "radial";

export interface GradientColorStop {
  offset: number;
  color: string;
}

export interface BgGradient {
  type: GradientType;
  colorStops: GradientColorStop[];
  x1?: number;
  y1?: number;
  x2?: number;
  y2?: number;
  r1?: number;
  r2?: number;
}

export interface MattingTask {
  id: string;
  fileName: string;
  filePath: string;
  thumbnail?: string;
  status: "idle" | "processing" | "completed" | "error";
  progress: number;
  error?: string;
  result?: MattingResult;
  settings: MattingSettings;
}

export interface MattingResult {
  outputPath: string;
  width: number;
  height: number;
  format: OutputFormat;
  fileSize: number;
  previewPath?: string;
  /** Mask image as PNG base64 data URL (grayscale, white=keep, black=remove) */
  maskDataUrl?: string;
}

export interface MattingSettings {
  mode: MattingMode;
  outputFormat: OutputFormat;
  quality: number;
  bgType: BgType;
  bgColor?: string;
  bgImageUrl?: string;
  bgGradient?: BgGradient;
  bgOpacity: number;
  targetWidth?: number;
  targetHeight?: number;
  maintainAspectRatio: boolean;
}

export interface AppSettings {
  outputDir: string;
  autoExport: boolean;
  autoOverwrite: boolean;
  defaultSettings: MattingSettings;
  modelPath?: string;
}

export interface ModelStatus {
  exists: boolean;
  path: string;
  size: number;
  downloading: boolean;
  progress: number;
  bytesDownloaded: number;
  totalBytes: number;
  speed: number;
  error?: string;
}

export const DEFAULT_SETTINGS: MattingSettings = {
  mode: "foreground",
  outputFormat: "png",
  quality: 95,
  bgType: "transparent",
  bgColor: "#ffffff",
  bgOpacity: 100,
  maintainAspectRatio: true,
};

export const DEFAULT_APP_SETTINGS: AppSettings = {
  outputDir: "",
  autoExport: true,
  autoOverwrite: false,
  defaultSettings: { ...DEFAULT_SETTINGS },
};
