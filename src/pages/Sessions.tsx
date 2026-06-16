import { useState } from "react";
import { useStore } from "../store/useStore";
import { useDb } from "../hooks/useDb";
import { JsonPreview } from "../components/JsonPreview";
import { Search, ChevronDown, ChevronUp, X, Code2, Layers, Zap, Activity } from "lucide-react";
import { DateFilter } from "../components/DateFilter";
import { projectName } from "../utils/project";
import { SkeletonTable, Skeleton } from "../components/Skeleton";
import { decodeModel } from "../utils/model";
import type { SessionRow, SessionDetail } from "../types";

function formatTime(ms: number) {
  const d = new Date(ms);
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
}

function formatDuration(start: number, end: number) {
  const diff = Math.floor((end - start) / 1000);
  if (diff < 60) return `${diff}s`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ${diff % 60}s`;
  return `${Math.floor(diff / 3600)}h ${Math.floor((diff % 3600) / 60)}m`;
}

function formatCost(c: number) {
  if (c >= 1_000_000_000_000) return `$${(c / 1_000_000_000_000).toFixed(2)}T`;
  if (c >= 1_000_000_000) return `$${(c / 1_000_000_000).toFixed(2)}B`;
  if (c >= 1_000_000) return `$${(c / 1_000_000).toFixed(2)}M`;
  if (c >= 1_000) return `$${(c / 1_000).toFixed(2)}K`;
  if (c >= 0.01) return `$${c.toFixed(2)}`;
  return `$${c.toFixed(4)}`;
}

function formatTokens(n: number) {
  if (n >= 1_000_000_000_000) return `${(n / 1_000_000_000_000).toFixed(1)}T`;
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function shortenId(id: string) {
  return id.length > 14 ? id.slice(0, 14) + "…" : id;
}

function SortIcon({ colKey, sortKey, sortAsc }: { colKey: keyof SessionRow; sortKey: keyof SessionRow; sortAsc: boolean }) {
  if (sortKey !== colKey) return null;
  return sortAsc ? <ChevronUp className="w-3 h-3 inline" /> : <ChevronDown className="w-3 h-3 inline" />;
}

export function Sessions() {
  const sessions = useStore((s) => s.sessions);
  const isLoading = useStore((s) => s.isLoading);
  const { fetchSessionDetail } = useDb();
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [sortKey, setSortKey] = useState<keyof SessionRow>("time_created");
  const [sortAsc, setSortAsc] = useState(false);
  const [showJson, setShowJson] = useState(false);

  if (isLoading && sessions.length === 0) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-5 w-24" />
        <SkeletonTable rows={8} />
      </div>
    );
  }

  const filtered = sessions
    .filter((s) => {
      if (!search) return true;
      const q = search.toLowerCase();
      return (
        s.id.toLowerCase().includes(q) ||
        (s.title?.toLowerCase().includes(q) ?? false) ||
        (s.model?.toLowerCase().includes(q) ?? false) ||
        (s.project_id?.toLowerCase().includes(q) ?? false)
      );
    })
    .sort((a, b) => {
      const aVal = a[sortKey] ?? 0;
      const bVal = b[sortKey] ?? 0;
      if (typeof aVal === "string" && typeof bVal === "string") {
        return sortAsc ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal);
      }
      return sortAsc ? Number(aVal) - Number(bVal) : Number(bVal) - Number(aVal);
    });

  function handleSort(key: keyof SessionRow) {
    if (sortKey === key) {
      setSortAsc(!sortAsc);
    } else {
      setSortKey(key);
      setSortAsc(false);
    }
  }

  async function handleSelect(id: string) {
    if (selectedId === id) {
      setSelectedId(null);
      setDetail(null);
      return;
    }
    setSelectedId(id);
    setShowJson(false);
    setLoadingDetail(true);
    try {
      const d = await fetchSessionDetail(id);
      setDetail(d);
    } catch {
      setDetail(null);
    } finally {
      setLoadingDetail(false);
    }
  }

  function prepareJsonData() {
    if (!detail) return null;
    return {
      id: detail.id,
      title: detail.title,
      model: detail.model,
      agent: detail.agent,
      cost: detail.cost,
      tokens: {
        input: detail.tokens_input,
        output: detail.tokens_output,
        reasoning: detail.tokens_reasoning,
        cache_read: detail.tokens_cache_read,
        cache_write: detail.tokens_cache_write,
      },
      summary: {
        additions: detail.summary_additions,
        deletions: detail.summary_deletions,
        files: detail.summary_files,
      },
      time_created: new Date(detail.time_created).toISOString(),
      time_updated: new Date(detail.time_updated).toISOString(),
      time_archived: detail.time_archived ? new Date(detail.time_archived).toISOString() : null,
      directory: detail.directory,
      project_id: detail.project_id,
      messages: detail.messages,
    };
  }

  return (
    <div className="space-y-4 animate-slide-in">
      <div className="flex items-center justify-between flex-wrap gap-2">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-bold">Sessions</h2>
          <span className="text-xs text-gray-500 bg-surface-700/50 px-2 py-0.5 rounded-full">{filtered.length} of {sessions.length}</span>
        </div>
        <div className="flex items-center gap-3">
          <DateFilter />
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-500" />
            <input
              type="text"
              placeholder="Search sessions..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-9 pr-3 py-2 bg-surface-800 border border-border rounded-lg text-sm focus:outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/20 w-48 placeholder:text-gray-600"
            />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4" style={{ gridTemplateColumns: selectedId ? "1fr 380px" : "1fr" }}>
        <div className="glass-card rounded-xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border text-gray-500 text-[11px] uppercase tracking-wider">
                  <th className="text-left p-3 cursor-pointer hover:text-gray-300 font-medium" onClick={() => handleSort("time_created")}>Date <SortIcon colKey="time_created" sortKey={sortKey} sortAsc={sortAsc} /></th>
                  <th className="text-left p-3 cursor-pointer hover:text-gray-300 font-medium" onClick={() => handleSort("title")}>Title <SortIcon colKey="title" sortKey={sortKey} sortAsc={sortAsc} /></th>
                  <th className="text-left p-3 cursor-pointer hover:text-gray-300 font-medium" onClick={() => handleSort("model")}>Model <SortIcon colKey="model" sortKey={sortKey} sortAsc={sortAsc} /></th>
                  <th className="text-right p-3 cursor-pointer hover:text-gray-300 font-medium" onClick={() => handleSort("tokens_input")}>Tokens <SortIcon colKey="tokens_input" sortKey={sortKey} sortAsc={sortAsc} /></th>
                  <th className="text-right p-3 cursor-pointer hover:text-gray-300 font-medium" onClick={() => handleSort("cost")}>Cost <SortIcon colKey="cost" sortKey={sortKey} sortAsc={sortAsc} /></th>
                  <th className="text-right p-3 font-medium">Duration</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((s) => (
                  <tr
                    key={s.id}
                    onClick={() => handleSelect(s.id)}
                    className={`border-b border-border/50 cursor-pointer transition-all hover:bg-surface-700/40 ${
                      selectedId === s.id ? "bg-accent-glow border-l-2 border-l-accent" : "border-l-2 border-l-transparent"
                    }`}
                  >
                    <td className="p-3 text-xs text-gray-400 whitespace-nowrap">
                      {formatTime(s.time_created)}
                    </td>
                    <td className="p-3 max-w-[180px]">
                      <div className="truncate text-gray-200">{s.title ?? "—"}</div>
                      <div className="text-[10px] text-gray-600 font-mono">{shortenId(s.id)}</div>
                    </td>
                    <td className="p-3">
                      <span className={`text-xs font-medium ${decodeModel(s.model).color}`}>{decodeModel(s.model).short}</span>
                      {decodeModel(s.model).tier && (
                        <span className={`ml-1.5 inline-flex items-center gap-0.5 px-1 py-0.5 rounded text-[9px] font-medium ${
                          decodeModel(s.model).tier === "high"
                            ? "bg-red-950/60 text-red-accent border border-red-900/50"
                            : "bg-yellow-950/60 text-yellow-accent border border-yellow-900/50"
                        }`}>
                          {decodeModel(s.model).tier === "high" ? <Zap className="w-2.5 h-2.5" /> : <Activity className="w-2.5 h-2.5" />}
                          {decodeModel(s.model).tier}
                        </span>
                      )}
                    </td>
                    <td className="p-3 text-right">
                      <span className="text-yellow-accent/90 font-medium text-xs">{formatTokens(s.tokens_input + s.tokens_output)}</span>
                    </td>
                    <td className="p-3 text-right">
                      <span className="text-green-accent/90 font-mono text-xs">{formatCost(s.cost)}</span>
                    </td>
                    <td className="p-3 text-right text-xs text-gray-500">{formatDuration(s.time_created, s.time_updated)}</td>
                  </tr>
                ))}
                {filtered.length === 0 && (
                  <tr>
                    <td colSpan={6} className="p-10 text-center text-gray-600">
                      <div className="space-y-1">
                        <p className="text-sm">No sessions found</p>
                        <p className="text-xs">Try adjusting your search or refresh the data</p>
                      </div>
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>

        {selectedId && (
          <div className="animate-slide-in">
            {loadingDetail ? (
              <div className="glass-card rounded-xl p-8 flex items-center justify-center">
                <div className="flex items-center gap-2 text-gray-500">
                  <div className="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
                  Loading...
                </div>
              </div>
            ) : detail ? (
              <div className="glass-card rounded-xl overflow-hidden">
                <div className="flex items-center justify-between p-4 border-b border-border">
                  <h3 className="font-semibold text-sm truncate pr-2">{detail.title ?? "Untitled"}</h3>
                  <button onClick={() => { setSelectedId(null); setDetail(null); }} className="p-1 hover:bg-surface-600 rounded transition-colors">
                    <X className="w-4 h-4 text-gray-500" />
                  </button>
                </div>

                <div className="flex border-b border-border">
                  <button
                    onClick={() => setShowJson(false)}
                    className={`flex-1 flex items-center justify-center gap-1.5 py-2.5 text-xs font-medium transition-colors ${
                      !showJson ? "text-accent border-b-2 border-accent bg-accent-glow" : "text-gray-500 hover:text-gray-300"
                    }`}
                  >
                    <Layers className="w-3.5 h-3.5" /> Details
                  </button>
                  <button
                    onClick={() => setShowJson(true)}
                    className={`flex-1 flex items-center justify-center gap-1.5 py-2.5 text-xs font-medium transition-colors ${
                      showJson ? "text-accent border-b-2 border-accent bg-accent-glow" : "text-gray-500 hover:text-gray-300"
                    }`}
                  >
                    <Code2 className="w-3.5 h-3.5" /> Raw JSON
                  </button>
                </div>

                {showJson ? (
                  <div className="p-4 max-h-[500px] overflow-auto">
                    <JsonPreview data={prepareJsonData()} collapsed={true} defaultCollapsedDepth={2} />
                  </div>
                ) : (
                  <div className="p-4 space-y-3 text-sm max-h-[500px] overflow-auto">
                    <div className="grid grid-cols-2 gap-x-4 gap-y-2">
                      <div>
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-0.5">Model</div>
                        <div className={decodeModel(detail.model).color}>{decodeModel(detail.model).display}</div>
                      </div>
                      <div>
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-0.5">Agent</div>
                        <div>{detail.agent ?? "—"}</div>
                      </div>
                      <div>
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-0.5">Cost</div>
                        <div className="text-green-accent font-mono">{formatCost(detail.cost)}</div>
                      </div>
                      <div>
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-0.5">Duration</div>
                        <div>{formatDuration(detail.time_created, detail.time_updated)}</div>
                      </div>
                      <div>
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-0.5">Created</div>
                        <div className="text-xs">{formatTime(detail.time_created)}</div>
                      </div>
                      <div>
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-0.5">Updated</div>
                        <div className="text-xs">{formatTime(detail.time_updated)}</div>
                      </div>
                    </div>

                    <div className="border-t border-border pt-3">
                      <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-2">Tokens</div>
                      <div className="grid grid-cols-2 gap-2">
                        {[
                          { label: "Input", value: detail.tokens_input, color: "text-accent" },
                          { label: "Output", value: detail.tokens_output, color: "text-green-accent" },
                          { label: "Reasoning", value: detail.tokens_reasoning, color: "text-yellow-accent" },
                          { label: "Cache Read", value: detail.tokens_cache_read, color: "text-purple-accent" },
                          { label: "Cache Write", value: detail.tokens_cache_write, color: "text-pink-accent" },
                        ].map((t) => (
                          <div key={t.label} className="bg-surface-700/50 rounded-lg p-2">
                            <div className="text-[10px] text-gray-500">{t.label}</div>
                            <div className={`font-mono text-xs font-medium ${t.color}`}>{formatTokens(t.value)}</div>
                          </div>
                        ))}
                      </div>
                    </div>

                    <div className="border-t border-border pt-3">
                      <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-2">Changes</div>
                      <div className="flex items-center gap-3">
                        <span className="text-green-accent text-xs font-mono">+{detail.summary_additions}</span>
                        <span className="text-red-accent text-xs font-mono">-{detail.summary_deletions}</span>
                        <span className="text-gray-400 text-xs">{detail.summary_files} files</span>
                      </div>
                    </div>

                    {detail.directory && (
                      <div className="border-t border-border pt-3">
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">Directory</div>
                        <div className="text-xs text-gray-400 font-mono break-all">{detail.directory}</div>
                      </div>
                    )}

                    {detail.project_id && (
                      <div className="border-t border-border pt-3">
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-1">Project</div>
                        <div className="text-xs text-accent">{projectName(detail.project_id, detail.directory)}</div>
                      </div>
                    )}

                    {detail.messages.length > 0 && (
                      <div className="border-t border-border pt-3">
                        <div className="text-[10px] text-gray-500 uppercase tracking-wider mb-2">Messages ({detail.messages.length})</div>
                        <div className="space-y-1">
                          {detail.messages.slice(0, 10).map((m) => (
                            <div key={m.id} className="bg-surface-700/30 rounded px-2 py-1.5 flex items-center justify-between text-xs">
                              <span className="text-gray-400 font-mono truncate max-w-[200px]">{m.id.slice(0, 20)}…</span>
                              <span className="text-gray-600">{m.parts_count} parts</span>
                            </div>
                          ))}
                          {detail.messages.length > 10 && (
                            <div className="text-[10px] text-gray-600 text-center pt-1">… and {detail.messages.length - 10} more</div>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            ) : (
              <div className="glass-card rounded-xl p-8 text-center text-gray-600 text-sm">Failed to load details</div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
