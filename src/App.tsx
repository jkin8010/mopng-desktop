import { useEffect, useCallback, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { useStore } from "@/store";
import { TitleBar } from "@/components/TitleBar";
import { DropZone } from "@/components/DropZone";
import { ThumbnailList } from "@/components/ThumbnailList";
import { PreviewCanvas } from "@/components/PreviewCanvas";
import { ControlPanel } from "@/components/ControlPanel";
import { TaskBar } from "@/components/TaskBar";
import { ModelDialog } from "@/components/ModelDialog";
import { SettingsDialog } from "@/components/SettingsDialog";
import { BatchProgress } from "@/components/BatchProgress";
import { Loader2 } from "lucide-react";
import type { MattingTask, ModelInfo } from "@/types";
import { generateId } from "@/lib/id";

function App() {
  const { tasks, selectedTaskId, addTasks, selectTask, updateTask, dragOver, setDragOver } =
    useStore();
  const { modelStatus, setModelStatus, setModelDialogOpen, availableModels, activeModelId, setAvailableModels, setActiveModelId } = useStore();
  const [initialized, setInitialized] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);

  const selectedTask = tasks.find((t) => t.id === selectedTaskId) || null;

  // 启动时加载模型列表并检查模型
  useEffect(() => {
    const initModels = async () => {
      try {
        // Scan models directory on startup to populate registry descriptors
        const models: ModelInfo[] = await invoke("scan_models");
        setAvailableModels(models);

        const modelId = activeModelId || "birefnet";
        if (!activeModelId) {
          setActiveModelId("birefnet");
        }

        // 检查当前选中的模型是否存在
        const info: { exists: boolean; path: string; size_bytes: number } = await invoke("check_model", { modelId });
        if (info.exists) {
          // 设置初始状态为 loading — 轮询会检测 loaded 状态
          setModelStatus({ exists: true, path: info.path, size: info.size_bytes, downloading: false, progress: 100, state: "loading" });

          // 非阻塞加载模型到内存（fire-and-forget）
          invoke("init_model", { modelId, modelPath: info.path }).catch((initErr) => {
            console.warn("模型加载到内存失败:", initErr);
          });
        } else {
          setModelStatus({ exists: false, downloading: false, state: "notDownloaded" });
          setModelDialogOpen(true);
        }
      } catch (err) {
        console.warn("模型检查失败:", err);
        setModelDialogOpen(true);
      } finally {
        setInitialized(true);
      }
    };
    initModels();
  }, []);

  // 轮询 list_models() 监控模型异步加载状态
  useEffect(() => {
    const pollInterval = setInterval(async () => {
      try {
        const models: ModelInfo[] = await invoke("list_models");
        setAvailableModels(models);
        const currentModel = models.find((m) => m.id === activeModelId);
        if (currentModel?.state === "loaded") {
          setModelStatus({ exists: true, downloading: false, progress: 100, state: "loaded" });
          if (useStore.getState().modelDialogOpen) {
            setModelDialogOpen(false);
          }
        } else if (currentModel?.state === "error") {
          console.warn("模型加载失败:", currentModel);
        }
      } catch {
        // 轮询失败静默处理
      }
    }, 500);
    return () => clearInterval(pollInterval);
  }, [activeModelId]);

  // 将本地文件路径转换为 Tauri asset URL
  const toAssetUrl = useCallback((path: string): string => {
    if (!path) return "";
    if (path.startsWith("data:")) return path;
    if (path.startsWith("http")) return path;
    try {
      return convertFileSrc(path);
    } catch {
      return path;
    }
  }, []);

  // Handle file drop from OS (tauri://drag-drop)
  useEffect(() => {
    const unlisten = listen<{ paths: string[] }>("tauri://drag-drop", (event) => {
      const paths = event.payload.paths;
      handleFiles(paths);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const handleFiles = useCallback(
    async (paths: string[]) => {
      const imagePaths = paths.filter((p) =>
        /\.(jpg|jpeg|png|webp|bmp|gif)$/i.test(p)
      );
      if (imagePaths.length === 0) return;

      const newTasks: MattingTask[] = imagePaths.map((path) => ({
        id: generateId(),
        fileName: path.split("/").pop() || path.split("\\").pop() || "unknown",
        filePath: path,
        status: "idle",
        progress: 0,
        settings: { ...useStore.getState().currentSettings },
      }));

      addTasks(newTasks);

      // 立即为每个任务生成预览图（使用 convertFileSrc 转换路径）
      for (const task of newTasks) {
        try {
          const assetUrl = toAssetUrl(task.filePath);
          updateTask(task.id, { thumbnail: assetUrl });
        } catch {
          // ignore
        }
      }

      // 后台生成高质量缩略图
      for (const task of newTasks) {
        try {
          const thumbnail = await invoke<string>("generate_thumbnail", {
            path: task.filePath,
            maxSize: 120,
          });
          updateTask(task.id, { thumbnail });
        } catch (err) {
          console.warn("缩略图生成失败:", task.filePath, err);
          // 保留 asset URL 作为回退
        }
      }
    },
    [addTasks, updateTask, toAssetUrl]
  );

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);

      const files = Array.from(e.dataTransfer.files);
      const paths = files.map((f: any) => f.path || f.name);
      handleFiles(paths);
    },
    [handleFiles, setDragOver]
  );

  const handleDragOver = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(true);
    },
    [setDragOver]
  );

  const handleDragLeave = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
    },
    [setDragOver]
  );

  if (!initialized) {
    return (
      <div className="flex flex-col items-center justify-center h-screen w-screen bg-background text-foreground">
        <div className="flex flex-col items-center gap-4">
          <div className="w-16 h-16 bg-primary/10 rounded-2xl flex items-center justify-center">
            <Loader2 className="w-8 h-8 text-primary animate-spin" />
          </div>
          <div className="text-center">
            <h1 className="text-lg font-semibold">模图桌面版</h1>
            <p className="text-sm text-muted-foreground mt-1">正在初始化...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div
      className="flex flex-col h-screen w-screen overflow-hidden bg-background text-foreground select-none"
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
    >
      <TitleBar />

      <main className="flex-1 flex overflow-hidden">
        {/* Left sidebar - thumbnails */}
        <ThumbnailList
          tasks={tasks}
          selectedId={selectedTaskId}
          onSelect={selectTask}
          onFilesSelected={handleFiles}
        />

        {/* Center - preview */}
        <div className="flex-1 flex flex-col relative">
          {dragOver && <DropZone />}

          <PreviewCanvas task={selectedTask} />

          {/* Bottom task bar */}
          <TaskBar task={selectedTask} />

          {/* Batch progress */}
          <BatchProgress />
        </div>

        {/* Right sidebar - controls */}
        <ControlPanel onOpenSettings={() => setSettingsOpen(true)} />
      </main>

      {/* Dialogs */}
      <ModelDialog />
      <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} />
    </div>
  );
}

export default App;
