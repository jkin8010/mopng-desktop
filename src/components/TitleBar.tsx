import { Image } from "lucide-react";

export function TitleBar() {
  return (
    <div className="h-8 flex items-center justify-between bg-background/80 backdrop-blur-sm border-b border-border/50 select-none px-3">
      <div className="flex items-center gap-2">
        <Image className="w-4 h-4 text-primary" />
        <span className="text-sm font-semibold">MoPNG Desktop</span>
        <span className="text-xs text-muted-foreground ml-1">模图桌面版</span>
      </div>

      {/* macOS 系统原生按钮在左边，这里只放右侧内容 */}
      <div className="flex items-center gap-2">
        <span className="text-xs text-muted-foreground">本地 AI 抠图</span>
      </div>
    </div>
  );
}
