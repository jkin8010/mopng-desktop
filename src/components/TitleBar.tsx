import { Image, Minus, Square, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function TitleBar() {
  const handleMinimize = async () => {
    const win = getCurrentWindow();
    await win.minimize();
  };

  const handleMaximize = async () => {
    const win = getCurrentWindow();
    const isMaximized = await win.isMaximized();
    if (isMaximized) {
      await win.unmaximize();
    } else {
      await win.maximize();
    }
  };

  const handleClose = async () => {
    const win = getCurrentWindow();
    await win.close();
  };

  return (
    <div
      data-tauri-drag-region
      className="h-10 flex items-center justify-between bg-muted/50 border-b border-border select-none"
    >
      <div className="flex items-center gap-2 px-3" data-tauri-drag-region>
        <Image className="w-5 h-5 text-primary" />
        <span className="text-sm font-semibold">MoPNG Desktop</span>
        <span className="text-xs text-muted-foreground ml-1">模图桌面版</span>
      </div>

      <div className="flex items-center" data-tauri-drag-region="false">
        <button
          onClick={handleMinimize}
          className="w-12 h-10 flex items-center justify-center hover:bg-muted transition-colors"
        >
          <Minus className="w-4 h-4" />
        </button>
        <button
          onClick={handleMaximize}
          className="w-12 h-10 flex items-center justify-center hover:bg-muted transition-colors"
        >
          <Square className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={handleClose}
          className="w-12 h-10 flex items-center justify-center hover:bg-destructive hover:text-destructive-foreground transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
