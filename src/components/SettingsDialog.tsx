import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useStore } from "@/store";
import {
  FolderOpen,
  HardDrive,
  RotateCcw,
  Settings2,
  Cpu,
  Info,
} from "lucide-react";
import { DEFAULT_APP_SETTINGS } from "@/types";
import { useEffect } from "react";

interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
  const { appSettings, modelStatus, updateAppSettings, setModelStatus } =
    useStore();
  const [activeTab, setActiveTab] = useState<"general" | "model" | "about">(
    "general"
  );

  // 监听模型下载进度
  useEffect(() => {
    if (!open) return;
    let unlistenProgress: UnlistenFn | undefined;
    let unlistenComplete: UnlistenFn | undefined;

    const setupListeners = async () => {
      unlistenProgress = await listen<{
        bytes_downloaded: number;
        total_bytes: number;
        percentage: number;
        speed_mbps: number;
        eta_seconds: number;
      }>("model-download-progress", (e) => {
        const { bytes_downloaded, total_bytes, percentage, speed_mbps } = e.payload;
        setModelStatus({
          downloading: true,
          progress: percentage,
          bytesDownloaded: bytes_downloaded,
          totalBytes: total_bytes,
          speed: speed_mbps,
        });
      });

      unlistenComplete = await listen<{
        exists: boolean;
        path: string;
        size_bytes: number;
      }>("model-download-complete", (e) => {
        const { exists, path, size_bytes } = e.payload;
        setModelStatus({
          exists,
          path,
          size: size_bytes,
          downloading: false,
          progress: 100,
        });
      });
    };

    setupListeners();
    return () => {
      unlistenProgress?.();
      unlistenComplete?.();
    };
  }, [open, setModelStatus]);

  const handleSelectOutputDir = useCallback(async () => {
    try {
      const result = await invoke<string | null>("select_output_dir");
      if (result) {
        updateAppSettings({ outputDir: result });
      }
    } catch (err) {
      console.warn("选择输出目录失败:", err);
    }
  }, [updateAppSettings]);

  const handleOpenModelDir = useCallback(async () => {
    try {
      const modelDir = await invoke<string>("get_model_dir");
      await invoke("open_in_folder", { path: modelDir });
    } catch (err) {
      console.warn("打开模型目录失败:", err);
    }
  }, []);

  const handleResetDefaults = useCallback(() => {
    updateAppSettings({ ...DEFAULT_APP_SETTINGS });
  }, [updateAppSettings]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl h-[560px] p-0 overflow-hidden">
        <DialogHeader className="px-6 pt-5 pb-3 border-b">
          <DialogTitle className="flex items-center gap-2 text-base">
            <Settings2 className="w-4 h-4" />
            偏好设置
          </DialogTitle>
        </DialogHeader>

        <div className="flex h-[calc(560px-64px)]">
          {/* 左侧标签 */}
          <div className="w-40 border-r bg-muted/30 flex flex-col py-2">
            {[
              { key: "general", label: "通用", icon: Settings2 },
              { key: "model", label: "模型", icon: Cpu },
              { key: "about", label: "关于", icon: Info },
            ].map((tab) => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key as typeof activeTab)}
                className={`flex items-center gap-2 px-4 py-2.5 text-sm transition-colors ${
                  activeTab === tab.key
                    ? "bg-accent text-accent-foreground font-medium"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <tab.icon className="w-4 h-4" />
                {tab.label}
              </button>
            ))}
          </div>

          {/* 右侧内容 */}
          <ScrollArea className="flex-1">
            <div className="p-6 space-y-6">
              {activeTab === "general" && (
                <>
                  {/* 输出目录 */}
                  <section className="space-y-3">
                    <h3 className="text-sm font-semibold">输出设置</h3>

                    <div className="space-y-2">
                      <Label className="text-xs text-muted-foreground">
                        默认输出目录
                      </Label>
                      <div className="flex gap-2">
                        <Input
                          value={appSettings.outputDir || "默认（应用数据目录）"}
                          readOnly
                          className="flex-1 text-xs"
                        />
                        <Button
                          variant="outline"
                          size="icon"
                          onClick={handleSelectOutputDir}
                          title="选择目录"
                        >
                          <FolderOpen className="w-4 h-4" />
                        </Button>
                      </div>
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label className="text-sm">自动导出</Label>
                        <p className="text-xs text-muted-foreground">
                          处理完成后自动保存到输出目录
                        </p>
                      </div>
                      <Switch
                        checked={appSettings.autoExport}
                        onCheckedChange={(v) =>
                          updateAppSettings({ autoExport: v })
                        }
                      />
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="space-y-0.5">
                        <Label className="text-sm">自动覆盖</Label>
                        <p className="text-xs text-muted-foreground">
                          输出文件已存在时直接覆盖
                        </p>
                      </div>
                      <Switch
                        checked={appSettings.autoOverwrite}
                        onCheckedChange={(v) =>
                          updateAppSettings({ autoOverwrite: v })
                        }
                      />
                    </div>
                  </section>

                  <div className="h-px bg-border" />

                  {/* 默认设置 */}
                  <section className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold">默认处理参数</h3>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={handleResetDefaults}
                        className="h-7 text-xs"
                      >
                        <RotateCcw className="w-3 h-3 mr-1" />
                        重置
                      </Button>
                    </div>

                    <div className="grid grid-cols-2 gap-3 text-xs">
                      <div className="bg-muted/50 rounded-lg p-3 space-y-1">
                        <p className="text-muted-foreground">模式</p>
                        <p className="font-medium">
                          {appSettings.defaultSettings.mode === "foreground"
                            ? "保留主体"
                            : "保留背景"}
                        </p>
                      </div>
                      <div className="bg-muted/50 rounded-lg p-3 space-y-1">
                        <p className="text-muted-foreground">格式</p>
                        <p className="font-medium uppercase">
                          {appSettings.defaultSettings.outputFormat}
                        </p>
                      </div>
                      <div className="bg-muted/50 rounded-lg p-3 space-y-1">
                        <p className="text-muted-foreground">质量</p>
                        <p className="font-medium">
                          {appSettings.defaultSettings.quality}%
                        </p>
                      </div>
                      <div className="bg-muted/50 rounded-lg p-3 space-y-1">
                        <p className="text-muted-foreground">背景</p>
                        <p className="font-medium">
                          {appSettings.defaultSettings.bgType === "transparent"
                            ? "透明"
                            : appSettings.defaultSettings.bgType === "white"
                            ? "白色"
                            : appSettings.defaultSettings.bgType === "color"
                            ? "纯色"
                            : "网格"}
                        </p>
                      </div>
                    </div>
                  </section>
                </>
              )}

              {activeTab === "model" && (
                <>
                  <section className="space-y-3">
                    <h3 className="text-sm font-semibold">模型管理</h3>

                    <div className="bg-muted/50 rounded-lg p-4 space-y-3">
                      <div className="flex items-center justify-between">
                        <span className="text-sm">模型状态</span>
                        <span
                          className={`text-xs px-2 py-0.5 rounded-full ${
                            modelStatus.exists
                              ? "bg-green-100 text-green-700"
                              : "bg-amber-100 text-amber-700"
                          }`}
                        >
                          {modelStatus.exists ? "已就绪" : "未下载"}
                        </span>
                      </div>

                      {modelStatus.exists && (
                        <>
                          <div className="space-y-1 text-xs">
                            <div className="flex justify-between">
                              <span className="text-muted-foreground">
                                路径
                              </span>
                              <span className="font-mono truncate max-w-[280px]">
                                {modelStatus.path}
                              </span>
                            </div>
                            <div className="flex justify-between">
                              <span className="text-muted-foreground">
                                大小
                              </span>
                              <span>{formatFileSize(modelStatus.size)}</span>
                            </div>
                          </div>

                          <Button
                            variant="outline"
                            size="sm"
                            className="w-full"
                            onClick={handleOpenModelDir}
                          >
                            <HardDrive className="w-3.5 h-3.5 mr-1.5" />
                            打开模型目录
                          </Button>
                        </>
                      )}

                      {modelStatus.downloading && (
                        <div className="space-y-2">
                          <div className="flex justify-between text-xs">
                            <span>下载中...</span>
                            <span>{modelStatus.progress.toFixed(1)}%</span>
                          </div>
                          <div className="h-1.5 bg-muted rounded-full overflow-hidden">
                            <div
                              className="h-full bg-primary transition-all duration-300"
                              style={{ width: `${modelStatus.progress}%` }}
                            />
                          </div>
                          <div className="flex justify-between text-xs text-muted-foreground">
                            <span>
                              {formatFileSize(modelStatus.bytesDownloaded)} /{" "}
                              {formatFileSize(modelStatus.totalBytes)}
                            </span>
                            <span>{modelStatus.speed.toFixed(1)} MB/s</span>
                          </div>
                        </div>
                      )}
                    </div>
                  </section>
                </>
              )}

              {activeTab === "about" && (
                <div className="space-y-4 text-center py-8">
                  <div className="w-16 h-16 bg-primary/10 rounded-xl mx-auto flex items-center justify-center">
                    <Settings2 className="w-8 h-8 text-primary" />
                  </div>
                  <div>
                    <h2 className="text-lg font-bold">模图桌面版</h2>
                    <p className="text-sm text-muted-foreground mt-1">
                      AI 智能抠图工具
                    </p>
                  </div>
                  <div className="text-xs text-muted-foreground space-y-1">
                    <p>版本 0.1.0</p>
                    <p>基于 BiRefNet 模型</p>
                    <p>Powered by Tauri + React</p>
                  </div>
                </div>
              )}
            </div>
          </ScrollArea>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(2) + " MB";
}
