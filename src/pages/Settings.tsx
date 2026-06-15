import { useState } from "react";
import { useStore } from "../store/useStore";
import { useDb } from "../hooks/useDb";
import { Save, Database, RefreshCw, Palette, Info } from "lucide-react";

function Section({ icon: Icon, title, desc, children }: { icon: React.ElementType; title: string; desc?: string; children: React.ReactNode }) {
  return (
    <div className="glass-card rounded-xl overflow-hidden">
      <div className="px-5 py-3.5 border-b border-border flex items-center gap-2.5">
        <Icon className="w-4 h-4 text-accent" />
        <div>
          <h3 className="text-sm font-semibold text-gray-200">{title}</h3>
          {desc && <p className="text-[11px] text-gray-500">{desc}</p>}
        </div>
      </div>
      <div className="p-5">{children}</div>
    </div>
  );
}

export function Settings() {
  const refreshInterval = useStore((s) => s.refreshInterval);
  const setRefreshInterval = useStore((s) => s.setRefreshInterval);
  const error = useStore((s) => s.error);
  const { refreshAll, setDbPath: setPath, setRefreshInterval: setInterval } = useDb();
  const [dbPath, setDbPath] = useState("");
  const [saved, setSaved] = useState(false);

  async function handleSaveDbPath() {
    if (!dbPath.trim()) return;
    await setPath(dbPath.trim());
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
    refreshAll();
  }

  async function handleRefreshChange(val: number) {
    setRefreshInterval(val);
    await setInterval(val);
  }

  return (
    <div className="space-y-5 max-w-2xl animate-slide-in">
      <h2 className="text-lg font-bold">Settings</h2>

      <Section icon={Database} title="Database Path" desc="Leave empty for auto-detect">
        <p className="text-xs text-gray-500 mb-3 leading-relaxed">
          The app searches <span className="text-accent font-mono text-[11px] bg-surface-700/50 px-1 py-0.5 rounded">~/.local/share/opencode/opencode.db</span>{" "}
          and <span className="text-accent font-mono text-[11px] bg-surface-700/50 px-1 py-0.5 rounded">%APPDATA%/opencode/opencode.db</span>.
        </p>
        <div className="flex gap-2">
          <input
            type="text"
            value={dbPath}
            onChange={(e) => setDbPath(e.target.value)}
            placeholder="C:\Users\...\opencode.db"
            className="flex-1 px-3 py-2 bg-surface-800 border border-border rounded-lg text-sm focus:outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/20 placeholder:text-gray-600"
          />
          <button
            onClick={handleSaveDbPath}
            className="flex items-center gap-2 px-4 py-2 bg-accent/20 hover:bg-accent/30 text-accent border border-accent/30 rounded-lg text-sm transition-all font-medium"
          >
            <Save className="w-4 h-4" />
            {saved ? "Saved!" : "Save"}
          </button>
        </div>
        {error && error.includes("database") && (
          <p className="text-xs text-red-accent mt-2">Current path not accessible: {error}</p>
        )}
      </Section>

      <Section icon={RefreshCw} title="Auto-Refresh" desc={`Data refreshes every ${refreshInterval} seconds`}>
        <input
          type="range"
          min={5}
          max={120}
          step={5}
          value={refreshInterval}
          onChange={(e) => handleRefreshChange(Number(e.target.value))}
          className="w-full accent-accent"
        />
        <div className="flex justify-between text-xs text-gray-600 mt-1">
          <span>5s</span>
          <span className="text-accent text-sm font-medium">{refreshInterval}s</span>
          <span>120s</span>
        </div>
      </Section>

      <Section icon={Palette} title="Theme">
        <select className="w-full px-3 py-2 bg-surface-800 border border-border rounded-lg text-sm focus:outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/20 text-gray-200" defaultValue="dark">
          <option value="dark">Dark</option>
          <option value="system">System</option>
          <option value="light">Light</option>
        </select>
      </Section>

      <Section icon={Info} title="About OpenCode Monitor">
        <div className="space-y-1 text-sm text-gray-400">
          <p>Version 0.1.0</p>
          <p className="text-xs text-gray-500">
            A Tauri desktop app for monitoring <span className="text-accent">opencode</span> AI coding agent usage.
          </p>
          <p className="text-xs text-gray-500">
            Reads data directly from the opencode SQLite database <span className="text-gray-400">(read-only)</span>.
          </p>
          <p className="text-xs text-gray-600 pt-2">Built with Tauri v2 + React + Rust</p>
        </div>
      </Section>
    </div>
  );
}
