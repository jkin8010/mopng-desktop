import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Play, FolderOpen, Download, Settings, Wrench } from "lucide-react";
import { useStore } from "@/store";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { MattingMode, OutputFormat, BgType } from "@/types";

interface ControlPanelProps {
  onOpenSettings: () => void;
}

export function ControlPanel({ onOpenSettings }: ControlPanelProps) {
  const {
    tasks,
    selectedTaskId,
    currentSettings,
    isProcessing,
    updateSettings,
    updateTask,
    updateTaskResult,
    setProcessing,
  } = useStore();

  const selectedTask = tasks.find((t) => t.id === selectedTaskId);

  const handleModeChange = (value: string) => {
    updateSettings({ mode: value as MattingMode });
  };

  const handleFormatChange = (value: string) => {
    updateSettings({ outputFormat: value as OutputFormat });
  };

  const handleBgTypeChange = (value: string) => {
    updateSettings({ bgType: value as BgType });
  };

  const handleQualityChange = (value: number[]) => {
    updateSettings({ quality: value[0] });
  };

  const handleProcess = useCallback(async () => {
    if (!selectedTask) return;

    updateTask(selectedTask.id, { status: "processing", progress: 0 });
    setProcessing(true);

    try {
      const result = await invoke<{
        outputPath: string;
        width: number;
        height: number;
        format: "png" | "jpg" | "webp";
        fileSize: number;
        previewPath: string;
      }>("process_image", {
        params: {
          filePath: selectedTask.filePath,
          settings: currentSettings,
        },
      });

      updateTaskResult(selectedTask.id, result);
    } catch (error) {
      updateTask(selectedTask.id, {
        status: "error",
        error: String(error),
      });
    } finally {
      setProcessing(false);
    }
  }, [selectedTask, currentSettings, updateTask, updateTaskResult, setProcessing]);

  const handleBatchProcess = useCallback(async () => {
    const pendingTasks = tasks.filter((t) => t.status === "idle");
    if (pendingTasks.length === 0) return;

    setProcessing(true);

    for (const task of pendingTasks) {
      updateTask(task.id, { status: "processing", progress: 0 });

      try {
        const result = await invoke<{
          outputPath: string;
          width: number;
          height: number;
          format: "png" | "jpg" | "webp";
          fileSize: number;
          previewPath: string;
        }>("process_image", {
          params: {
            filePath: task.filePath,
            settings: task.settings,
          },
        });

        updateTaskResult(task.id, result);
      } catch (error) {
        updateTask(task.id, {
          status: "error",
          error: String(error),
        });
      }
    }

    setProcessing(false);
  }, [tasks, updateTask, updateTaskResult, setProcessing]);

  const handleOpenOutput = useCallback(async () => {
    if (!selectedTask?.result?.outputPath) return;
    await invoke("open_in_folder", { path: selectedTask.result.outputPath });
  }, [selectedTask]);

  const handleExport = useCallback(async () => {
    if (!selectedTask?.result?.outputPath) return;
    await invoke("export_image", {
      sourcePath: selectedTask.result.outputPath,
    });
  }, [selectedTask]);

  return (
    <TooltipProvider>
      <div className="w-72 flex flex-col border-l border-border bg-card overflow-y-auto">
        <div className="p-4 border-b border-border flex items-center justify-between">
          <h2 className="text-sm font-semibold flex items-center gap-2">
            <Wrench className="w-4 h-4" />
            处理设置
          </h2>
          <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onOpenSettings} title="偏好设置">
            <Settings className="w-4 h-4" />
          </Button>
        </div>

        <div className="p-4 space-y-5">
          {/* Mode */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-muted-foreground">抠图模式</label>
            <Select value={currentSettings.mode} onValueChange={handleModeChange}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="foreground">保留主体（透明背景）</SelectItem>
                <SelectItem value="background">保留背景（移除主体）</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Output Format */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-muted-foreground">输出格式</label>
            <Select value={currentSettings.outputFormat} onValueChange={handleFormatChange}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="png">PNG (支持透明)</SelectItem>
                <SelectItem value="jpg">JPG</SelectItem>
                <SelectItem value="webp">WebP</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Background Type */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-muted-foreground">背景类型</label>
            <Select value={currentSettings.bgType} onValueChange={handleBgTypeChange}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="transparent">透明</SelectItem>
                <SelectItem value="white">白色</SelectItem>
                <SelectItem value="color">纯色</SelectItem>
                <SelectItem value="checkerboard">网格</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Quality */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <label className="text-xs font-medium text-muted-foreground">输出质量</label>
              <span className="text-xs font-mono">{currentSettings.quality}%</span>
            </div>
            <Slider
              value={[currentSettings.quality]}
              onValueChange={handleQualityChange}
              min={10}
              max={100}
              step={1}
            />
          </div>

          <div className="h-px bg-border" />

          {/* Actions */}
          <div className="space-y-2">
            <Button
              className="w-full"
              onClick={handleProcess}
              disabled={!selectedTask || isProcessing || selectedTask.status === "processing"}
            >
              <Play className="w-4 h-4 mr-2" />
              {selectedTask?.status === "processing" ? "处理中..." : "开始抠图"}
            </Button>

            <Button
              variant="outline"
              className="w-full"
              onClick={handleBatchProcess}
              disabled={isProcessing}
            >
              <Play className="w-4 h-4 mr-2" />
              批量处理 ({tasks.filter((t) => t.status === "idle").length})
            </Button>
          </div>

          {selectedTask?.result && (
            <>
              <div className="h-px bg-border" />
              <div className="space-y-2">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="outline" className="w-full" onClick={handleOpenOutput}>
                      <FolderOpen className="w-4 h-4 mr-2" />
                      打开所在文件夹
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>在文件管理器中打开</p>
                  </TooltipContent>
                </Tooltip>

                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="outline" className="w-full" onClick={handleExport}>
                      <Download className="w-4 h-4 mr-2" />
                      另存为...
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>导出到指定位置</p>
                  </TooltipContent>
                </Tooltip>
              </div>

              <div className="text-xs text-muted-foreground space-y-1">
                <p>输出信息:</p>
                <p>尺寸: {selectedTask.result.width} x {selectedTask.result.height}</p>
                <p>大小: {formatFileSize(selectedTask.result.fileSize)}</p>
                <p>格式: {selectedTask.result.format.toUpperCase()}</p>
              </div>
            </>
          )}
        </div>
      </div>
    </TooltipProvider>
  );
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(2) + " MB";
}
