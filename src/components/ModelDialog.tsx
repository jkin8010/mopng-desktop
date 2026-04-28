import { useEffect, useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { useStore } from "@/store";
import { AlertCircle, Download, CheckCircle, XCircle, Loader2 } from "lucide-react";
import type { ModelSource } from "@/types";

const MODEL_SIZE_MB = 900;

interface DownloadProgressEvent {
  bytes_downloaded: number;
  total_bytes: number;
  percentage: number;
  speed_mbps: number;
  eta_seconds: number;
}

interface ModelCompleteEvent {
  exists: boolean;
  path: string;
  size_bytes: number;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatSpeed(mbps: number): string {
  if (mbps < 1.0) return `${(mbps * 1024).toFixed(0)} KB/s`;
  return `${mbps.toFixed(1)} MB/s`;
}

function formatETA(seconds: number): string {
  if (seconds < 60) return `${seconds}秒`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}分${seconds % 60}秒`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return `${h}小时${m}分`;
}

export function ModelDialog() {
  const { modelStatus, setModelStatus, setModelDialogOpen, modelDialogOpen, activeModelId, availableModels } = useStore();
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const [sources, setSources] = useState<ModelSource[]>([]);
  const [selectedSourceId, setSelectedSourceId] = useState<string>("mocdn");

  const activeModel = availableModels.find((m) => m.id === activeModelId);
  const modelDisplayName = activeModel?.name ?? "AI 模型";
  const modelFilename = activeModel?.filename ?? "model.onnx";

  // Listen for download progress events
  useEffect(() => {
    let mounted = true;
    let unlistenProgress: UnlistenFn | null = null;
    let unlistenComplete: UnlistenFn | null = null;

    const setupListeners = async () => {
      unlistenProgress = await listen<DownloadProgressEvent>(
        "model-download-progress",
        (event) => {
          if (!mounted) return;
          const { bytes_downloaded, total_bytes, percentage, speed_mbps } = event.payload;
          setModelStatus({
            ...useStore.getState().modelStatus,
            bytesDownloaded: bytes_downloaded,
            totalBytes: total_bytes,
            progress: percentage,
            speed: speed_mbps,
            downloading: true,
          });
        }
      );

      unlistenComplete = await listen<ModelCompleteEvent>(
        "model-download-complete",
        (event) => {
          if (!mounted) return;
          const { exists, path, size_bytes } = event.payload;
          // 加载模型到内存
          invoke("init_model", { modelId: activeModelId, modelPath: path }).catch(
            (e) => console.warn("初始化模型失败:", e)
          );
          setModelStatus({
            ...useStore.getState().modelStatus,
            exists,
            path,
            size: size_bytes,
            downloading: false,
            progress: 100,
            error: undefined,
          });
        }
      );
    };

    setupListeners();

    return () => {
      mounted = false;
      unlistenProgress?.();
      unlistenComplete?.();
    };
  }, [setModelStatus]);

  // 加载可用下载源
  useEffect(() => {
    invoke<ModelSource[]>("get_model_sources", { modelId: activeModelId }).then((srcs) => {
      setSources(srcs);
      const def = srcs.find((s) => s.default);
      if (def) setSelectedSourceId(def.id);
    }).catch(() => {
      // ignore - fallback to manual selection
    });
  }, []);

  const handleDownload = useCallback(async () => {
    const src = sources.find((s) => s.id === selectedSourceId);
    const url = src?.url;
    if (!url) return;

    try {
      setModelStatus({
        ...useStore.getState().modelStatus,
        downloading: true,
        error: undefined,
        progress: 0,
      });

      const path = await invoke<string>("download_model", { sourceUrl: url, modelId: activeModelId });

      // 加载模型到内存
      try {
        await invoke("init_model", { modelId: activeModelId, modelPath: path });
      } catch (initErr: any) {
        setModelStatus({
          ...useStore.getState().modelStatus,
          downloading: false,
          error: `模型初始化失败: ${initErr?.message || initErr}`,
        });
        return;
      }

      setModelStatus({
        ...useStore.getState().modelStatus,
        exists: true,
        path,
        downloading: false,
        progress: 100,
        error: undefined,
      });
    } catch (err: any) {
      const msg = err?.message || "";
      // 取消下载不显示错误
      if (msg === "下载已取消") {
        setModelStatus({
          ...useStore.getState().modelStatus,
          downloading: false,
        });
      } else {
        setModelStatus({
          ...useStore.getState().modelStatus,
          downloading: false,
          error: msg || "下载失败",
        });
      }
    }
  }, [setModelStatus, sources, selectedSourceId]);

  const handleCancel = useCallback(async () => {
    try {
      await invoke("cancel_download", { modelId: activeModelId });
    } catch {
      // ignore
    }
    setModelStatus({
      ...useStore.getState().modelStatus,
      downloading: false,
    });
  }, [setModelStatus]);

  const handleClose = useCallback(() => {
    // Only allow close if model exists or not downloading
    if (!modelStatus.downloading) {
      setModelDialogOpen(false);
    }
  }, [modelStatus.downloading, setModelDialogOpen]);

  // 如果模型已存在且对话框是开着的，自动关闭
  useEffect(() => {
    if (modelStatus.exists && modelDialogOpen) {
      const timer = setTimeout(() => setModelDialogOpen(false), 800);
      return () => clearTimeout(timer);
    }
  }, [modelStatus.exists, modelDialogOpen, setModelDialogOpen]);

  return (
    <Dialog open={modelDialogOpen} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md" hideClose={modelStatus.downloading}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {modelStatus.exists ? (
              <CheckCircle className="w-5 h-5 text-green-500" />
            ) : (
              <AlertCircle className="w-5 h-5 text-amber-500" />
            )}
            {modelStatus.exists ? "模型就绪" : "需要下载 AI 模型"}
          </DialogTitle>
          <DialogDescription>
            {modelStatus.exists
              ? `${modelDisplayName} 模型已就绪，可以开始使用`
              : `首次使用需要下载 ${modelDisplayName} 模型（约 ${MODEL_SIZE_MB} MB）`}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {modelStatus.exists && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <CheckCircle className="w-4 h-4 text-green-500" />
              <span>模型路径: {modelStatus.path}</span>
            </div>
          )}

          {!modelStatus.exists && !modelStatus.downloading && !modelStatus.error && (
            <div className="space-y-3">
              <div className="bg-muted rounded-lg p-4 text-sm space-y-2">
                <div className="flex items-center gap-2">
                  <Download className="w-4 h-4" />
                  <span>模型: {modelFilename} (FP32)</span>
                </div>
                <div className="text-muted-foreground">
                  大小: ~{MODEL_SIZE_MB} MB
                </div>
              </div>

              {/* 源选择 */}
              {sources.length > 0 && (
                <div className="space-y-2">
                  <label className="text-sm font-medium">选择下载源</label>
                  <div className="grid gap-2">
                    {sources.map((src) => (
                      <button
                        key={src.id}
                        onClick={() => setSelectedSourceId(src.id)}
                        className={`flex items-start gap-3 rounded-lg border p-3 text-left text-sm transition-colors ${
                          selectedSourceId === src.id
                            ? "border-primary bg-primary/5"
                            : "border-border hover:bg-muted"
                        }`}
                      >
                        <input
                          type="radio"
                          checked={selectedSourceId === src.id}
                          onChange={() => setSelectedSourceId(src.id)}
                          className="mt-0.5"
                        />
                        <div className="flex-1 min-w-0">
                          <div className="font-medium">{src.name}</div>
                          <div className="text-muted-foreground truncate">
                            {src.description}
                          </div>
                        </div>
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {modelStatus.downloading && (
            <div className="space-y-3 py-2">
              {/* 进度条 */}
              <Progress value={modelStatus.progress} className="h-2.5" />

              {modelStatus.bytesDownloaded === 0 ? (
                <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground py-4">
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span>准备中...</span>
                </div>
              ) : (
                <>
              {/* 主要数据行 */}
              <div className="flex justify-between items-center text-sm">
                <span className="font-medium text-foreground">
                  {modelStatus.progress.toFixed(1)}%
                </span>
                <span className="text-muted-foreground">
                  {formatBytes(modelStatus.bytesDownloaded)} / {formatBytes(modelStatus.totalBytes)}
                </span>
              </div>

              {/* 速度和 ETA */}
              {modelStatus.speed > 0 && (
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span className="flex items-center gap-1">
                    <Loader2 className="w-3 h-3 animate-spin" />
                    {formatSpeed(modelStatus.speed)}
                  </span>
                  {modelStatus.totalBytes > 0 && (
                    <span>
                      剩余时间: {formatETA(
                        Math.max(
                          0,
                          Math.floor(
                            (modelStatus.totalBytes - modelStatus.bytesDownloaded) / (modelStatus.speed * 1024 * 1024)
                          )
                        )
                      )}
                    </span>
                  )}
                </div>
              )}

              {/* 断点续传提示 */}
              {modelStatus.bytesDownloaded > 0 && modelStatus.progress < 100 && (
                <div className="text-xs text-muted-foreground bg-muted/50 rounded px-2 py-1">
                  支持断点续传，关闭应用后再次下载将从上次进度继续
                </div>
              )}
              </>
            )}
            </div>
          )}

          {modelStatus.error && (
            <div className="flex items-center gap-2 text-sm text-destructive bg-destructive/10 rounded-lg p-3">
              <XCircle className="w-4 h-4 shrink-0" />
              <span>{modelStatus.error}</span>
            </div>
          )}

          <div className="flex gap-2 pt-2">
            {!modelStatus.exists && !modelStatus.downloading && (
              <Button onClick={handleDownload} className="flex-1">
                <Download className="w-4 h-4 mr-2" />
                开始下载
              </Button>
            )}

            {modelStatus.downloading && (
              <Button onClick={handleCancel} variant="outline" className="flex-1">
                取消下载
              </Button>
            )}

            {modelStatus.exists && (
              <Button onClick={handleClose} className="flex-1">
                开始使用
              </Button>
            )}

            {!modelStatus.downloading && !modelStatus.exists && (
              <Button variant="ghost" onClick={handleClose}>
                稍后再说
              </Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
