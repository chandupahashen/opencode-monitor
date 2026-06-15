import { useState } from "react";
import { Minus, Square, X, Monitor } from "lucide-react";

export function Titlebar() {
  const [maximized, setMaximized] = useState(false);

  async function exec(method: "minimize" | "toggleMaximize" | "close") {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      if (method === "minimize") await win.minimize();
      else if (method === "toggleMaximize") {
        await win.toggleMaximize();
        setMaximized(await win.isMaximized());
      } else await win.close();
    } catch {
      // not in Tauri context
    }
  }

  return (
    <div className="flex items-center justify-between h-9 bg-surface-900 select-none flex-shrink-0" data-tauri-drag-region>
      <div className="flex items-center gap-2 px-3" data-tauri-drag-region>
        <div className="w-5 h-5 rounded-md bg-gradient-to-br from-accent to-cyan-600 flex items-center justify-center flex-shrink-0">
          <Monitor className="w-3 h-3 text-white" />
        </div>
        <span className="text-xs font-semibold text-gray-300" data-tauri-drag-region>openCode Monitor</span>
      </div>
      <div className="flex">
        <button
          onClick={() => exec("minimize")}
          className="w-10 h-9 flex items-center justify-center hover:bg-surface-700 transition-colors text-gray-400 hover:text-gray-200"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={() => exec("toggleMaximize")}
          className="w-10 h-9 flex items-center justify-center hover:bg-surface-700 transition-colors text-gray-400 hover:text-gray-200"
        >
          <Square className={`w-3 h-3 ${maximized ? "rotate-180" : ""}`} />
        </button>
        <button
          onClick={() => exec("close")}
          className="w-10 h-9 flex items-center justify-center hover:bg-red-500 hover:text-white transition-colors text-gray-400"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
