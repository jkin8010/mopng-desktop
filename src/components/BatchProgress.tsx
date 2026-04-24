import { useStore } from "@/store";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  CheckCircle2,
  AlertCircle,
  Loader2,
  X,
  RotateCcw,
  Trash2,
} from "lucide-react";

export function BatchProgress() {
  const { tasks, isProcessing, clearCompleted } = useStore();

  const processing = tasks.filter((t) => t.status === "processing");
  const completed = tasks.filter((t) => t.status === "completed");
  const errors = tasks.filter((t) => t.status === "error");
  const idle = tasks.filter((t) => t.status === "idle");

  const total = tasks.length;
  const done = completed.length + errors.length;
  const progress = total > 0 ? (done / total) * 100 : 0;

  if (total === 0) return null;

  return (
    <div className="border-t bg-card">
      {/* 汇总栏 */}
      <div className="flex items-center gap-4 px-4 py-2 border-b bg-muted/30">
        <div className="flex-1 flex items-center gap-3">
          {isProcessing && <Loader2 className="w-3.5 h-3.5 animate-spin text-primary" />}
          <span className="text-xs font-medium">
            {isProcessing
              ? `处理中 ${done}/${total}`
              : `已完成 ${done}/${total}`}
          </span>
          <div className="flex-1 max-w-[200px]">
            <Progress value={progress} className="h-1.5" />
          </div>
        </div>

        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
          {completed.length > 0 && (
            <span className="flex items-center gap-0.5 text-green-600">
              <CheckCircle2 className="w-3 h-3" />
              {completed.length}
            </span>
          )}
          {errors.length > 0 && (
            <span className="flex items-center gap-0.5 text-red-500">
              <AlertCircle className="w-3 h-3" />
              {errors.length}
            </span>
          )}
          {idle.length > 0 && !isProcessing && (
            <span className="flex items-center gap-0.5 text-amber-500">
              <RotateCcw className="w-3 h-3" />
              {idle.length}
            </span>
          )}
        </div>

        {completed.length > 0 && (
          <Button
            variant="ghost"
            size="sm"
            className="h-6 text-xs px-2"
            onClick={clearCompleted}
          >
            <Trash2 className="w-3 h-3 mr-1" />
            清理已完成
          </Button>
        )}
      </div>

      {/* 任务列表 */}
      {errors.length > 0 && (
        <ScrollArea className="max-h-[120px]">
          <div className="px-4 py-2 space-y-1">
            {errors.map((task) => (
              <div
                key={task.id}
                className="flex items-center gap-2 text-xs text-red-500 bg-red-50 rounded px-2 py-1"
              >
                <AlertCircle className="w-3 h-3 shrink-0" />
                <span className="truncate flex-1">{task.fileName}</span>
                <span className="text-red-400 truncate max-w-[200px]">
                  {task.error}
                </span>
              </div>
            ))}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
