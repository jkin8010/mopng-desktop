import { Trash2, FolderOpen, Download } from "lucide-react";
import { useStore } from "@/store";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import type { MattingTask } from "@/types";

interface TaskBarProps {
  task: MattingTask | null;
}

export function TaskBar({ task }: TaskBarProps) {
  const { tasks, clearCompleted, clearAll } = useStore();

  const completedCount = tasks.filter((t) => t.status === "completed").length;
  const processingCount = tasks.filter((t) => t.status === "processing").length;

  const handleOpenOutput = async () => {
    if (!task?.result?.outputPath) return;
    await invoke("open_in_folder", { path: task.result.outputPath });
  };

  const handleExport = async () => {
    if (!task?.result?.outputPath) return;
    await invoke("export_image_dialog", {
      sourcePath: task.result.outputPath,
    });
  };

  return (
    <div className="h-12 flex items-center justify-between px-4 border-t border-border bg-card">
      <div className="flex items-center gap-4 text-xs text-muted-foreground">
        <span>共 {tasks.length} 张</span>
        {processingCount > 0 && (
          <span className="text-primary">
            处理中 {processingCount} 张
          </span>
        )}
        {completedCount > 0 && (
          <span className="text-green-500">
            已完成 {completedCount} 张
          </span>
        )}
      </div>

      <div className="flex items-center gap-2">
        {task?.result && (
          <>
            <Button variant="ghost" size="sm" onClick={handleOpenOutput}>
              <FolderOpen className="w-4 h-4 mr-1" />
              打开
            </Button>
            <Button variant="ghost" size="sm" onClick={handleExport}>
              <Download className="w-4 h-4 mr-1" />
              导出
            </Button>
          </>
        )}

        <div className="w-px h-5 bg-border mx-1" />

        {completedCount > 0 && (
          <Button variant="ghost" size="sm" onClick={clearCompleted}>
            <Trash2 className="w-4 h-4 mr-1" />
            清除已完成
          </Button>
        )}

        {tasks.length > 0 && (
          <Button variant="ghost" size="sm" onClick={clearAll}>
            <Trash2 className="w-4 h-4 mr-1" />
            清除全部
          </Button>
        )}
      </div>
    </div>
  );
}
