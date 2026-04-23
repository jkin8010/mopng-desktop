import { Upload } from "lucide-react";

export function DropZone() {
  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
      <div className="drop-active flex flex-col items-center gap-4 p-12 rounded-2xl border-2 border-dashed border-primary bg-primary/5">
        <Upload className="w-16 h-16 text-primary" />
        <div className="text-center">
          <p className="text-xl font-semibold text-primary">释放以添加图片</p>
          <p className="text-sm text-muted-foreground mt-1">
            支持 JPG、PNG、WebP 格式
          </p>
        </div>
      </div>
    </div>
  );
}
