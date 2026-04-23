import { useState, useRef, useEffect, useCallback } from "react";
import { ZoomIn, ZoomOut, RotateCcw, ImageOff, Loader2 } from "lucide-react";
import type { MattingTask } from "@/types";
import { cn } from "@/lib/utils";

interface PreviewCanvasProps {
  task: MattingTask | null;
}

export function PreviewCanvas({ task }: PreviewCanvasProps) {
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [showOriginal, setShowOriginal] = useState(false);
  const canvasRef = useRef<HTMLDivElement>(null);
  const imgRef = useRef<HTMLImageElement>(null);

  // Auto-fit on task change
  useEffect(() => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  }, [task?.id]);

  const handleWheel = useCallback((e: React.WheelEvent) => {
    if (!e.ctrlKey && !e.metaKey) return;
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.1 : 0.1;
    setZoom((z) => Math.max(0.1, Math.min(5, z + delta)));
  }, []);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    setIsDragging(true);
    setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
  }, [pan]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDragging) return;
    setPan({
      x: e.clientX - dragStart.x,
      y: e.clientY - dragStart.y,
    });
  }, [isDragging, dragStart]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  if (!task) {
    return (
      <div className="flex-1 flex items-center justify-center bg-muted/30">
        <div className="text-center text-muted-foreground">
          <ImageOff className="w-16 h-16 mx-auto mb-4 opacity-50" />
          <p className="text-lg font-medium">暂无图片</p>
          <p className="text-sm mt-1">拖拽图片到此处或点击左侧添加</p>
        </div>
      </div>
    );
  }

  const imageSrc = showOriginal
    ? task.filePath
    : task.result?.previewPath || task.result?.outputPath || task.filePath;

  return (
    <div
      ref={canvasRef}
      className="flex-1 relative overflow-hidden bg-muted/30 cursor-grab active:cursor-grabbing"
      onWheel={handleWheel}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      {/* Checkerboard background */}
      <div
        className={cn(
          "absolute inset-0",
          task.settings.bgType === "checkerboard" && "checkerboard"
        )}
        style={
          task.settings.bgType === "color" && task.settings.bgColor
            ? { backgroundColor: task.settings.bgColor }
            : task.settings.bgType === "white"
            ? { backgroundColor: "#ffffff" }
            : undefined
        }
      />

      {/* Image */}
      <div
        className="absolute inset-0 flex items-center justify-center"
        style={{
          transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
          transformOrigin: "center",
        }}
      >
        {task.status === "processing" && !task.result ? (
          <div className="flex flex-col items-center gap-3">
            <Loader2 className="w-12 h-12 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground">AI 抠图中...</p>
          </div>
        ) : (
          <img
            ref={imgRef}
            src={imageSrc}
            alt={task.fileName}
            className="max-w-full max-h-full object-contain"
            draggable={false}
          />
        )}
      </div>

      {/* Toolbar */}
      <div className="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-1 p-1.5 rounded-lg bg-background/80 backdrop-blur-sm border border-border shadow-lg">
        <button
          onClick={() => setZoom((z) => Math.max(0.1, z - 0.2))}
          className="p-2 rounded-md hover:bg-muted transition-colors"
          title="缩小"
        >
          <ZoomOut className="w-4 h-4" />
        </button>
        <span className="text-xs font-mono w-14 text-center">
          {Math.round(zoom * 100)}%
        </span>
        <button
          onClick={() => setZoom((z) => Math.min(5, z + 0.2))}
          className="p-2 rounded-md hover:bg-muted transition-colors"
          title="放大"
        >
          <ZoomIn className="w-4 h-4" />
        </button>
        <div className="w-px h-5 bg-border mx-1" />
        <button
          onClick={() => {
            setZoom(1);
            setPan({ x: 0, y: 0 });
          }}
          className="p-2 rounded-md hover:bg-muted transition-colors"
          title="重置视图"
        >
          <RotateCcw className="w-4 h-4" />
        </button>
        {task.result && (
          <>
            <div className="w-px h-5 bg-border mx-1" />
            <button
              onMouseDown={() => setShowOriginal(true)}
              onMouseUp={() => setShowOriginal(false)}
              onMouseLeave={() => setShowOriginal(false)}
              className="px-3 py-1.5 text-xs rounded-md hover:bg-muted transition-colors"
              title="按住查看原图"
            >
              按住对比
            </button>
          </>
        )}
      </div>

      {/* Status indicator */}
      {task.status === "processing" && (
        <div className="absolute top-4 left-1/2 -translate-x-1/2 flex items-center gap-2 px-4 py-2 rounded-full bg-primary text-primary-foreground text-sm shadow-lg">
          <Loader2 className="w-4 h-4 animate-spin" />
          处理中... {task.progress}%
        </div>
      )}

      {task.status === "error" && (
        <div className="absolute top-4 left-1/2 -translate-x-1/2 px-4 py-2 rounded-full bg-destructive text-destructive-foreground text-sm shadow-lg">
          错误: {task.error}
        </div>
      )}
    </div>
  );
}
