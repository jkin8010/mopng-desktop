import { useCallback, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Play, FolderOpen, Download, Settings, Wrench, ImagePlus, Lock, Unlock } from "lucide-react";
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
import { GradientAnglePicker, angleToCoords, coordsToAngle } from "@/components/GradientAnglePicker";
import { Switch } from "@/components/ui/switch";
import { ScrubInput } from "@/components/ui/scrub-input";
import { SIZE_TEMPLATES, deriveTemplateId } from "@/types";
import type { MattingMode, OutputFormat, BgType, SizeTemplateId } from "@/types";

interface ControlPanelProps {
  onOpenSettings: () => void;
}

export function ControlPanel({ onOpenSettings }: ControlPanelProps) {
  const {
    tasks,
    selectedTaskId,
    currentSettings,
    isProcessing,
    availableModels,
    activeModelId,
    updateSettings,
    updateTask,
    updateTaskResult,
    selectTask,
    setActiveModelId,
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

  const handleOpacityChange = (value: number[]) => {
    updateSettings({ bgOpacity: value[0] });
  };

  const [selectedTemplateId, setSelectedTemplateId] = useState<SizeTemplateId>(
    deriveTemplateId(currentSettings),
  );
  const [aspectLocked, setAspectLocked] = useState(true);
  const aspectRatioRef = useRef(4 / 3);

  const handleTemplateChange = (templateId: string) => {
    const id = templateId as SizeTemplateId;
    setSelectedTemplateId(id);
    if (id === "original") {
      updateSettings({ targetWidth: undefined, targetHeight: undefined });
    } else if (id === "custom") {
      if (currentSettings.targetWidth == null || currentSettings.targetHeight == null) {
        updateSettings({ targetWidth: 800, targetHeight: 600 });
      }
      aspectRatioRef.current = (currentSettings.targetWidth ?? 800) / (currentSettings.targetHeight ?? 600);
    } else {
      const tpl = SIZE_TEMPLATES.find((t) => t.id === id);
      if (tpl) {
        updateSettings({ targetWidth: tpl.width, targetHeight: tpl.height });
      }
    }
  };

  const handlePickBgImage = useCallback(async () => {
    try {
      const selected = await open({
        filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "bmp"] }],
        multiple: false,
      });
      if (selected) {
        const url = convertFileSrc(selected);
        updateSettings({ bgImageUrl: url });
      }
    } catch (e) {
      console.error("Failed to pick background image:", e);
    }
  }, [updateSettings]);

  const handleGradientAngleChange = (angle: number) => {
    const g = currentSettings.bgGradient;
    const stops = g?.colorStops ?? [
      { offset: 0, color: "#000000" },
      { offset: 1, color: "#ffffff" },
    ];
    const { x1, y1, x2, y2 } = angleToCoords(angle);
    updateSettings({
      bgGradient: {
        type: "linear",
        colorStops: stops,
        x1,
        y1,
        x2,
        y2,
      },
    });
  };

  const handleGradientStartColor = (color: string) => {
    const stops = currentSettings.bgGradient?.colorStops ?? [
      { offset: 0, color: "#000000" },
      { offset: 1, color: "#ffffff" },
    ];
    stops[0] = { offset: 0, color };
    const g = currentSettings.bgGradient;
    updateSettings({
      bgGradient: {
        type: "linear",
        colorStops: stops,
        x1: g?.x1 ?? 0,
        y1: g?.y1 ?? 0,
        x2: g?.x2 ?? 1,
        y2: g?.y2 ?? 0,
      },
    });
  };

  const handleGradientEndColor = (color: string) => {
    const stops = currentSettings.bgGradient?.colorStops ?? [
      { offset: 0, color: "#000000" },
      { offset: 1, color: "#ffffff" },
    ];
    stops[1] = { offset: 1, color };
    const g = currentSettings.bgGradient;
    updateSettings({
      bgGradient: {
        type: "linear",
        colorStops: stops,
        x1: g?.x1 ?? 0,
        y1: g?.y1 ?? 0,
        x2: g?.x2 ?? 1,
        y2: g?.y2 ?? 0,
      },
    });
  };

  const currentAngle = (() => {
    const g = currentSettings.bgGradient;
    if (g?.x1 != null && g?.y1 != null && g?.x2 != null && g?.y2 != null) {
      return Math.round(coordsToAngle(g.x1, g.y1, g.x2, g.y2));
    }
    return 0;
  })();

  const handleProcess = useCallback(async () => {
    if (!selectedTask) return;

    updateTask(selectedTask.id, {
      status: "processing",
      progress: 0,
      settings: { ...currentSettings },
    });
    setProcessing(true);

    try {
      const result = await invoke<{
        outputPath: string;
        width: number;
        height: number;
        format: "png" | "jpg" | "webp";
        fileSize: number;
        previewPath: string;
        maskDataUrl?: string;
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
    const total = pendingTasks.length;

    for (let i = 0; i < total; i++) {
      const task = pendingTasks[i];

      selectTask(task.id);
      updateTask(task.id, { status: "processing", progress: 0 });
      useStore.getState().setGlobalProgress(Math.round((i / total) * 100));

      try {
        const result = await invoke<{
          outputPath: string;
          width: number;
          height: number;
          format: "png" | "jpg" | "webp";
          fileSize: number;
          previewPath: string;
          maskDataUrl?: string;
        }>("process_image", {
          params: {
            filePath: task.filePath,
            settings: task.settings,
          },
        });

        updateTaskResult(task.id, result);
        useStore.getState().setGlobalProgress(Math.round(((i + 1) / total) * 100));
      } catch (error) {
        updateTask(task.id, {
          status: "error",
          error: String(error),
        });
      }
    }

    setProcessing(false);
  }, [tasks, updateTask, updateTaskResult, selectTask, setProcessing]);

  const handleOpenOutput = useCallback(async () => {
    if (!selectedTask?.result?.outputPath) return;
    await invoke("open_in_folder", { path: selectedTask.result.outputPath });
  }, [selectedTask]);

  const handleExport = useCallback(async () => {
    if (!selectedTask?.result?.outputPath) return;
    const state = useStore.getState();
    const isTransparent = state.currentSettings.bgType === "transparent";
    const exportFn = state.konvaExportFn;

    console.log("[export] bgType=", state.currentSettings.bgType, "exportFn=", !!exportFn);

    try {
      if (isTransparent || !exportFn) {
        console.log("[export] falling back to export_image_dialog (transparent or no engine)");
        const result = await invoke<string>("export_image_dialog", {
          sourcePath: selectedTask.result.outputPath,
        });
        console.log("[export] export_image_dialog result:", result);
      } else {
        const ext = state.currentSettings.outputFormat;
        const suggestedName = selectedTask.fileName.replace(/\.[^.]+$/, "") + "_matting." + ext;
        const mimeType = ext === "jpg" ? "image/jpeg" : `image/${ext}`;
        const quality = ext === "png" ? undefined : 95;
        let dataUrl: string;
        try {
          dataUrl = exportFn(mimeType, quality) || "";
        } catch (fnErr) {
          console.error("[export] Konva exportFn threw:", fnErr);
          return;
        }
        console.log("[export] dataUrl length:", dataUrl.length, "mimeType:", mimeType);
        if (!dataUrl) {
          console.warn("[export] empty dataUrl from Konva export");
          return;
        }
        const result = await invoke<string>("save_data_url", {
          dataUrl,
          suggestedName,
        });
        console.log("[export] saved to:", result);
      }
    } catch (e) {
      console.error("[export] failed:", e);
    }
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
          {/* Model Selector */}
          {availableModels.length > 1 && (
            <div className="space-y-2">
              <label className="text-xs font-medium text-muted-foreground">AI 模型</label>
              <Select value={activeModelId} onValueChange={setActiveModelId}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {availableModels.map((m) => (
                    <SelectItem key={m.id} value={m.id}>
                      {m.name}
                      {!m.loaded && <span className="ml-2 text-xs text-muted-foreground">(未下载)</span>}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

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
                <SelectItem value="image">图片</SelectItem>
                <SelectItem value="gradient">渐变</SelectItem>
              </SelectContent>
            </Select>

            {currentSettings.bgType === "color" && (
              <div className="flex items-center gap-2 pt-1">
                <input
                  type="color"
                  value={currentSettings.bgColor || "#ffffff"}
                  onChange={(e) => updateSettings({ bgColor: e.target.value })}
                  className="w-8 h-8 p-0.5 rounded cursor-pointer border border-border bg-transparent"
                />
                <span className="text-xs font-mono text-muted-foreground">
                  {currentSettings.bgColor || "#ffffff"}
                </span>
              </div>
            )}

            {currentSettings.bgType === "image" && (
              <div className="space-y-2 pt-1">
                <Button
                  variant="outline"
                  size="sm"
                  className="w-full"
                  onClick={handlePickBgImage}
                >
                  <ImagePlus className="w-3.5 h-3.5 mr-1.5" />
                  选择背景图片
                </Button>
                {currentSettings.bgImageUrl && (
                  <p className="text-xs text-muted-foreground truncate">
                    已选择背景图片
                  </p>
                )}
              </div>
            )}

            {currentSettings.bgType === "gradient" && (
              <div className="space-y-3 pt-1">
                <div className="space-y-1.5">
                  <label className="text-xs text-muted-foreground">方向</label>
                  <GradientAnglePicker
                    angleDeg={currentAngle}
                    onAngleChange={handleGradientAngleChange}
                  />
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <div className="space-y-1">
                    <label className="text-xs text-muted-foreground">起始色</label>
                    <div className="flex items-center gap-1.5">
                      <input
                        type="color"
                        value={currentSettings.bgGradient?.colorStops?.[0]?.color ?? "#000000"}
                        onChange={(e) => handleGradientStartColor(e.target.value)}
                        className="w-7 h-7 p-0.5 rounded cursor-pointer border border-border bg-transparent"
                      />
                    </div>
                  </div>
                  <div className="space-y-1">
                    <label className="text-xs text-muted-foreground">结束色</label>
                    <div className="flex items-center gap-1.5">
                      <input
                        type="color"
                        value={currentSettings.bgGradient?.colorStops?.[1]?.color ?? "#ffffff"}
                        onChange={(e) => handleGradientEndColor(e.target.value)}
                        className="w-7 h-7 p-0.5 rounded cursor-pointer border border-border bg-transparent"
                      />
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Background Opacity */}
          {(currentSettings.bgType === "color" ||
            currentSettings.bgType === "image" ||
            currentSettings.bgType === "gradient") && (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium text-muted-foreground">背景不透明度</label>
                <span className="text-xs font-mono">{currentSettings.bgOpacity}%</span>
              </div>
              <Slider
                value={[currentSettings.bgOpacity]}
                onValueChange={handleOpacityChange}
                min={0}
                max={100}
                step={1}
              />
            </div>
          )}

          {/* Size Template */}
          <div className="space-y-2">
            <label className="text-xs font-medium text-muted-foreground">尺寸模板</label>
            <Select value={selectedTemplateId} onValueChange={handleTemplateChange}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SIZE_TEMPLATES.map((t) => (
                  <SelectItem key={t.id} value={t.id}>
                    {t.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            {selectedTemplateId === "custom" && (
              <div className="grid items-end gap-1 pt-1" style={{ gridTemplateColumns: "1fr auto 1fr" }}>
                <div className="space-y-1">
                  <label className="text-xs text-muted-foreground">宽度 (px)</label>
                  <ScrubInput
                    value={currentSettings.targetWidth ?? 800}
                    onChange={(v) => {
                      setSelectedTemplateId("custom");
                      if (aspectLocked) {
                        updateSettings({ targetWidth: v, targetHeight: Math.max(1, Math.round(v / aspectRatioRef.current)) });
                      } else {
                        aspectRatioRef.current = v / (currentSettings.targetHeight ?? 600);
                        updateSettings({ targetWidth: v });
                      }
                    }}
                    min={1}
                    max={10000}
                    className="h-8 text-xs"
                  />
                </div>
                <button
                  type="button"
                  className="flex items-center justify-center w-7 h-8 rounded hover:bg-muted transition-colors text-muted-foreground"
                  title={aspectLocked ? "解锁宽高比" : "锁定宽高比"}
                  onClick={() => {
                    const w = currentSettings.targetWidth ?? 800;
                    const h = currentSettings.targetHeight ?? 600;
                    aspectRatioRef.current = w / h;
                    setAspectLocked(!aspectLocked);
                  }}
                >
                  {aspectLocked ? <Lock className="w-3.5 h-3.5" /> : <Unlock className="w-3.5 h-3.5" />}
                </button>
                <div className="space-y-1">
                  <label className="text-xs text-muted-foreground">高度 (px)</label>
                  <ScrubInput
                    value={currentSettings.targetHeight ?? 600}
                    onChange={(v) => {
                      setSelectedTemplateId("custom");
                      if (aspectLocked) {
                        updateSettings({ targetHeight: v, targetWidth: Math.max(1, Math.round(v * aspectRatioRef.current)) });
                      } else {
                        aspectRatioRef.current = (currentSettings.targetWidth ?? 800) / v;
                        updateSettings({ targetHeight: v });
                      }
                    }}
                    min={1}
                    max={10000}
                    className="h-8 text-xs"
                  />
                </div>
              </div>
            )}

            {selectedTemplateId !== "original" && (
              <div className="flex items-center gap-2 pt-1">
                <Switch
                  id="maintain-ar"
                  checked={currentSettings.maintainAspectRatio}
                  onCheckedChange={(v) => updateSettings({ maintainAspectRatio: v })}
                />
                <label
                  htmlFor="maintain-ar"
                  className="text-xs text-muted-foreground cursor-pointer select-none"
                >
                  保持宽高比 (等比缩放居中)
                </label>
              </div>
            )}
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
              {isProcessing
                ? `批量处理中...`
                : `批量处理 (${tasks.filter((t) => t.status === "idle").length})`}
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
