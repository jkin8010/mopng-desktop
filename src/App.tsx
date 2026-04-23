import { useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "@/store";
import { TitleBar } from "@/components/TitleBar";
import { DropZone } from "@/components/DropZone";
import { ThumbnailList } from "@/components/ThumbnailList";
import { PreviewCanvas } from "@/components/PreviewCanvas";
import { ControlPanel } from "@/components/ControlPanel";
import { TaskBar } from "@/components/TaskBar";
import type { MattingTask } from "@/types";
import { generateId } from "@/lib/id";

function App() {
  const { tasks, selectedTaskId, addTasks, selectTask, dragOver, setDragOver } =
    useStore();

  const selectedTask = tasks.find((t) => t.id === selectedTaskId) || null;

  // Handle file drop from OS
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

      // Generate thumbnails
      for (const task of newTasks) {
        try {
          const thumbnail = await invoke<string>("generate_thumbnail", {
            path: task.filePath,
            maxSize: 120,
          });
          useStore.getState().updateTask(task.id, { thumbnail });
        } catch {
          // thumbnail generation failed, ignore
        }
      }
    },
    [addTasks]
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
        />

        {/* Center - preview */}
        <div className="flex-1 flex flex-col relative">
          {dragOver && <DropZone />}

          <PreviewCanvas task={selectedTask} />

          {/* Bottom task bar */}
          <TaskBar task={selectedTask} />
        </div>

        {/* Right sidebar - controls */}
        <ControlPanel />
      </main>
    </div>
  );
}

export default App;
