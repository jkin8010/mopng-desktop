import { useEffect, useCallback, useRef } from "react";
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

const MODEL_SIZE_MB = 178;

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatSpeed(bps: number): string {
  if (bps < 1024) return `${bps.toFixed(0)} B/s`;
  if (bps < 1024 * 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${(bps / (1024 * 1024)).toFixed(1)} MB/s`;
}

export function ModelDialog() {
  const { modelStatus, setModelStatus, setModelDialogOpen, modelDialogOpen } = useStore();
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // Listen for download progress events
  useEffect(() => {
    let mounted = true;

    const setupListener = async () => {
      const unlisten = await listen<{
        bytesDownloaded: number;
        totalBytes: number;
        bytesPerSecond: number;
      }>("download-progress", (event) => {
        if (!mounted) return;
        const { bytesDownloaded, totalBytes, bytesPerSecond } = event.payload;
        setModelStatus({
          ...useStore.getState().modelStatus,
          bytesDownloaded,
          totalBytes,
          progress: totalBytes > 0 ? (bytesDownloaded / totalBytes) * 100 : 0,
          speed: bytesPerSecond,
          downloading: true,
        });
      });

      unlistenRef.current = unlisten;
    };

    setupListener();

    return () => {
      mounted = false;
      unlistenRef.current?.();
    };
  }, [setModelStatus]);

  const handleDownload = useCallback(async () => {
    try {
      setModelStatus({
        ...useStore.getState().modelStatus,
        downloading: true,
        error: undefined,
        progress: 0,
      });

      const result = await invoke<{
        success: boolean;
        path: string;
        error?: string;
      }>("download_model", {});

      if (result.success) {
        setModelStatus({
          ...useStore.getState().modelStatus,
          exists: true,
          downloading: false,
          progress: 100,
          path: result.path,
        });
      } else {
        setModelStatus({
          ...useStore.getState().modelStatus,
          downloading: false,
          error: result.error || "下载失败",
        });
      }
    } catch (err: any) {
      setModelStatus({
        ...useStore.getState().modelStatus,
        downloading: false,
        error: err?.message || "下载失败",
      });
    }
  }, [setModelStatus]);

  const handleCancel = useCallback(async () => {
    try {
      await invoke("cancel_download", {});
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
              ? "BiRefNet 模型已就绪，可以开始使用"
              : `首次使用需要下载 BiRefNet 模型（约 ${MODEL_SIZE_MB} MB）`}
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
            <div className="bg-muted rounded-lg p-4 text-sm space-y-2">
              <div className="flex items-center gap-2">
                <Download className="w-4 h-4" />
                <span>模型: birefnet.onnx</span>
              </div>
              <div className="text-muted-foreground">
                大小: ~{MODEL_SIZE_MB} MB · 来源: HuggingFace
              </div>
            </div>
          )}

          {modelStatus.downloading && (
            <div className="space-y-2">
              <Progress value={modelStatus.progress} className="h-2" />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>{formatBytes(modelStatus.bytesDownloaded)} / {formatBytes(modelStatus.totalBytes)}</span>
                <span>{modelStatus.progress.toFixed(1)}%</span>
              </div>
              {modelStatus.speed > 0 && (
                <div className="text-xs text-muted-foreground">
                  速度: {formatSpeed(modelStatus.speed)}
                </div>
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
