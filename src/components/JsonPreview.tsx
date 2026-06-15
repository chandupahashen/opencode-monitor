import { useState } from "react";
import { ChevronRight, ChevronDown, Copy, Check } from "lucide-react";

interface JsonPreviewProps {
  data: unknown;
  collapsed?: boolean;
  maxDepth?: number;
  defaultCollapsedDepth?: number;
}

export function JsonPreview({ data, collapsed = false, maxDepth = 20, defaultCollapsedDepth = 3 }: JsonPreviewProps) {
  const [copied, setCopied] = useState(false);

  function handleCopy() {
    navigator.clipboard.writeText(JSON.stringify(data, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <div className="relative group">
      <button
        onClick={handleCopy}
        className="absolute top-2 right-2 p-1.5 rounded-md bg-surface-600/50 opacity-0 group-hover:opacity-100 hover:bg-surface-500 transition-all z-10"
        title="Copy JSON"
      >
        {copied ? <Check className="w-3.5 h-3.5 text-green-accent" /> : <Copy className="w-3.5 h-3.5 text-gray-400" />}
      </button>
      <div className="font-mono text-xs leading-relaxed">
        <JsonNode value={data} depth={0} maxDepth={maxDepth} defaultCollapsedDepth={defaultCollapsedDepth} initialCollapsed={collapsed} />
      </div>
    </div>
  );
}

function JsonNode({ value, depth, maxDepth, defaultCollapsedDepth, initialCollapsed }: { value: unknown; depth: number; maxDepth: number; defaultCollapsedDepth: number; initialCollapsed: boolean }) {
  const [isCollapsed, setIsCollapsed] = useState(depth >= defaultCollapsedDepth && initialCollapsed);

  if (depth > maxDepth) {
    return <span className="text-gray-500">...</span>;
  }

  if (value === null) {
    return <span className="text-gray-400">null</span>;
  }

  if (typeof value === "boolean") {
    return <span className="text-yellow-accent">{value ? "true" : "false"}</span>;
  }

  if (typeof value === "number") {
    return <span className="text-orange-accent">{String(value)}</span>;
  }

  if (typeof value === "string") {
    return (
      <span>
        <span className="text-green-accent/80">"</span>
        <span className="text-green-accent/90">{escapeString(value)}</span>
        <span className="text-green-accent/80">"</span>
      </span>
    );
  }

  if (Array.isArray(value)) {
    if (value.length === 0) {
      return <span className="text-gray-500">[]</span>;
    }

    if (isCollapsed) {
      return (
        <span className="inline-flex items-center gap-1 cursor-pointer hover:bg-surface-600/50 rounded px-0.5" onClick={() => setIsCollapsed(false)}>
          <ChevronRight className="w-3 h-3 text-gray-500" />
          <span className="text-gray-400">Array({value.length})</span>
        </span>
      );
    }

    return (
      <div>
        <span className="inline-flex items-center gap-1 cursor-pointer hover:bg-surface-600/50 rounded px-0.5" onClick={() => setIsCollapsed(true)}>
          <ChevronDown className="w-3 h-3 text-gray-500" />
          <span className="text-gray-500">[</span>
        </span>
        <div className="pl-4 border-l border-border/50 ml-1.5">
          {value.map((item, i) => (
            <div key={i} className="py-0.5">
              <span className="text-gray-500 mr-1">{i}:</span>
              <JsonNode value={item} depth={depth + 1} maxDepth={maxDepth} defaultCollapsedDepth={defaultCollapsedDepth} initialCollapsed={initialCollapsed} />
              {i < value.length - 1 && <span className="text-gray-600">,</span>}
            </div>
          ))}
        </div>
        <span className="text-gray-500">]</span>
      </div>
    );
  }

  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) {
      return <span className="text-gray-500">{ }</span>;
    }

    if (isCollapsed) {
      return (
        <span className="inline-flex items-center gap-1 cursor-pointer hover:bg-surface-600/50 rounded px-0.5" onClick={() => setIsCollapsed(false)}>
          <ChevronRight className="w-3 h-3 text-gray-500" />
          <span className="text-gray-400">Object({entries.length})</span>
        </span>
      );
    }

    return (
      <div>
        <span className="inline-flex items-center gap-1 cursor-pointer hover:bg-surface-600/50 rounded px-0.5" onClick={() => setIsCollapsed(true)}>
          <ChevronDown className="w-3 h-3 text-gray-500" />
          <span className="text-gray-500">{ }</span>
        </span>
        <div className="pl-4 border-l border-border/50 ml-1.5">
          {entries.map(([key, val], i) => (
            <div key={key} className="py-0.5">
              <span className="text-accent mr-1">"{key}"</span>
              <span className="text-gray-600 mr-1">: </span>
              <JsonNode value={val} depth={depth + 1} maxDepth={maxDepth} defaultCollapsedDepth={defaultCollapsedDepth} initialCollapsed={initialCollapsed} />
              {i < entries.length - 1 && <span className="text-gray-600">,</span>}
            </div>
          ))}
        </div>
        <span className="text-gray-500">{ }</span>
      </div>
    );
  }

  return <span>{String(value)}</span>;
}

function escapeString(s: string): string {
  return s
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t")
    .slice(0, 500) + (s.length > 500 ? "..." : "");
}
