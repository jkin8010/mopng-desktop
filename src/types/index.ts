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

export type SizeTemplateId = "original" | "一寸" | "小二寸" | "二寸" | "大一寸" | "五寸" | "custom";

export interface SizeTemplate {
  id: SizeTemplateId;
  label: string;
  width: number;
  height: number;
}

export const SIZE_TEMPLATES: SizeTemplate[] = [
  { id: "original", label: "原始尺寸", width: 0, height: 0 },
  { id: "一寸", label: "一寸 (25x35mm)", width: 295, height: 413 },
  { id: "小二寸", label: "小二寸 (33x48mm)", width: 390, height: 567 },
  { id: "二寸", label: "二寸 (35x49mm)", width: 413, height: 579 },
  { id: "大一寸", label: "大一寸 (33x48mm)", width: 390, height: 567 },
  { id: "五寸", label: '五寸 (5x3.5")', width: 1500, height: 1050 },
  { id: "custom", label: "自定义", width: 0, height: 0 },
];

export function deriveTemplateId(s: MattingSettings): SizeTemplateId {
  if (s.targetWidth == null && s.targetHeight == null) return "original";
  const exact = SIZE_TEMPLATES.find(t => t.width === s.targetWidth && t.height === s.targetHeight);
  if (exact) return exact.id;
  return "custom";
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
  state: "notDownloaded" | "loading" | "loaded" | "error";
  error?: string;
}

export interface ModelSource {
  id: string;
  name: string;
  description: string;
  url: string;
  default: boolean;
}

export interface SourceError {
  source_id: string;
  source_name: string;
  error_type: string;
  detail: string;
}

export interface DownloadErrorResponse {
  message: string;
  source_errors: SourceError[];
  model_filename: string;
}

export interface PluginCapabilities {
  matting: boolean;
  backgroundReplace: boolean;
  edgeRefinement: boolean;
  uncertaintyMask: boolean;
}

export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  state: "notDownloaded" | "loading" | "loaded" | "error";
  checksum?: string;
  filename: string;
  sources: ModelSource[];
  paramSchema?: Record<string, unknown>;
  capabilities?: PluginCapabilities;
}

/** Per-model parameter values keyed by parameter name (e.g., { threshold: 0.5 }) */
export type ModelParams = Record<string, unknown>;

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
