import { X, ImagePlus, Loader2 } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "@/store";
import type { MattingTask } from "@/types";
import { cn } from "@/lib/utils";

interface ThumbnailListProps {
  tasks: MattingTask[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onFilesSelected?: (paths: string[]) => void;
}

export function ThumbnailList({ tasks, selectedId, onSelect, onFilesSelected }: ThumbnailListProps) {
  const removeTask = useStore((s) => s.removeTask);

  const handleFileSelect = async () => {
    const files = await open({
      multiple: true,
      filters: [{
        name: "图片",
        extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"],
      }],
    });

    // open() returns string[] | null in Tauri v2
    if (files && files.length > 0) {
      onFilesSelected?.(files);
    }
  };

  return (
    <div className="w-48 flex flex-col border-r border-border bg-card">
      <div className="p-3 border-b border-border">
        <button
          onClick={handleFileSelect}
          className="w-full flex items-center justify-center gap-2 py-2 px-3 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors text-sm font-medium"
        >
          <ImagePlus className="w-4 h-4" />
          添加图片
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {tasks.length === 0 && (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <ImagePlus className="w-8 h-8 mb-2 opacity-50" />
            <p className="text-xs">暂无图片</p>
            <p className="text-xs">拖拽或点击添加</p>
          </div>
        )}

        {tasks.map((task) => (
          <div
            key={task.id}
            onClick={() => onSelect(task.id)}
            className={cn(
              "group relative flex items-center gap-2 p-1.5 rounded-md cursor-pointer transition-colors",
              selectedId === task.id
                ? "bg-primary/10 border border-primary/30"
                : "hover:bg-muted border border-transparent"
            )}
          >
            <div className="relative w-10 h-10 rounded overflow-hidden bg-muted flex-shrink-0">
              {task.thumbnail ? (
                <img
                  src={task.thumbnail}
                  alt={task.fileName}
                  className="w-full h-full object-cover"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center">
                  <ImagePlus className="w-4 h-4 text-muted-foreground" />
                </div>
              )}
              {task.status === "processing" && (
                <div className="absolute inset-0 bg-background/60 flex items-center justify-center">
                  <Loader2 className="w-4 h-4 animate-spin text-primary" />
                </div>
              )}
              {task.status === "completed" && (
                <div className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2 border-card" />
              )}
              {task.status === "error" && (
                <div className="absolute bottom-0 right-0 w-3 h-3 bg-destructive rounded-full border-2 border-card" />
              )}
            </div>

            <div className="flex-1 min-w-0">
              <p className="text-xs truncate font-medium">{task.fileName}</p>
              <p className="text-[10px] text-muted-foreground truncate">
                {task.status === "processing" && `${task.progress}%`}
                {task.status === "completed" && "已完成"}
                {task.status === "error" && "失败"}
                {task.status === "idle" && "待处理"}
              </p>
            </div>

            <button
              onClick={(e) => {
                e.stopPropagation();
                removeTask(task.id);
              }}
              className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-destructive/10 hover:text-destructive transition-all"
            >
              <X className="w-3 h-3" />
            </button>
          </div>
        ))}
      </div>

      {tasks.length > 0 && (
        <div className="p-2 border-t border-border text-[10px] text-muted-foreground text-center">
          共 {tasks.length} 张图片
        </div>
      )}
    </div>
  );
}
