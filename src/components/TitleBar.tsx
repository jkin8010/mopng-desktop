import { Image } from "lucide-react";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function TitleBar() {
  const [platform, setPlatform] = useState<"macos" | "windows" | "linux" | "unknown">("unknown");

  useEffect(() => {
    const ua = navigator.userAgent.toLowerCase();
    if (ua.includes("mac")) setPlatform("macos");
    else if (ua.includes("win")) setPlatform("windows");
    else if (ua.includes("linux")) setPlatform("linux");
  }, []);

  const handleMouseDown = (e: React.MouseEvent) => {
    // 只有左键点击才触发拖拽
    if (e.button === 0) {
      getCurrentWindow().startDragging();
    }
  };

  // macOS: 左侧留出 ~80px 安全区（红绿灯按钮）
  // Windows/Linux: 右侧留出 ~140px 安全区（最小化/最大化/关闭按钮）
  const leftPadding = platform === "macos" ? "pl-20" : "pl-3";
  const rightPadding = platform === "windows" || platform === "linux" ? "pr-36" : "pr-3";

  return (
    <div
      className={`h-8 flex items-center justify-between bg-background/80 backdrop-blur-sm border-b border-border/50 select-none ${leftPadding} ${rightPadding}`}
      onMouseDown={handleMouseDown}
    >
      <div className="flex items-center gap-2">
        <Image className="w-4 h-4 text-primary" />
        <span className="text-sm font-semibold">MoPNG Desktop</span>
        <span className="text-xs text-muted-foreground ml-1">模图桌面版</span>
      </div>

      <div className="flex items-center gap-2">
        <span className="text-xs text-muted-foreground">本地 AI 抠图</span>
      </div>
    </div>
  );
}
