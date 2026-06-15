import { useStore } from "../store/useStore";
import { FolderOpen, Folder } from "lucide-react";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";
import { DateFilter } from "../components/DateFilter";
import { SkeletonCard, Skeleton } from "../components/Skeleton";
import { projectDir } from "../utils/project";

function formatTokens(n: number) {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function formatCost(c: number) {
  if (c < 0.01) return `$${c.toFixed(4)}`;
  return `$${c.toFixed(2)}`;
}

const tooltipStyle = { background: "#1a1a25", border: "1px solid #2a2a3e", borderRadius: 8, fontSize: 12, boxShadow: "0 4px 20px rgba(0,0,0,0.4)" };

export function Projects() {
  const projectStats = useStore((s) => s.projectStats);
  const isLoading = useStore((s) => s.isLoading);

  if (isLoading && projectStats.length === 0) {
    return (
      <div className="space-y-5">
        <Skeleton className="h-5 w-24" />
        <SkeletonCard />
        <div className="grid gap-3">
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="glass-card rounded-xl p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Skeleton className="w-5 h-5 rounded" />
                  <div>
                    <Skeleton className="h-4 w-32 mb-1" />
                    <Skeleton className="h-3 w-20" />
                  </div>
                </div>
                <div className="flex items-center gap-5">
                  {Array.from({ length: 3 }).map((_, j) => (
                    <div key={j}>
                      <Skeleton className="h-2 w-10 mb-1" />
                      <Skeleton className="h-3 w-12" />
                    </div>
                  ))}
                </div>
              </div>
              <Skeleton className="h-1.5 w-full rounded-full mt-3" />
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (projectStats.length === 0) {
    return (
      <div className="space-y-4 animate-slide-in">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-bold">Projects</h2>
          <DateFilter />
        </div>
        <div className="glass-card rounded-xl flex items-center justify-center h-64">
          <div className="text-center space-y-2">
            <FolderOpen className="w-8 h-8 mx-auto text-gray-600" />
            <p className="text-gray-500 text-sm">No project data available</p>
          </div>
        </div>
      </div>
    );
  }

  const chartData = projectStats.slice(0, 15).map((p) => ({
    name: p.project_name,
    tokens: p.total_tokens,
    cost: p.total_cost,
    sessions: p.session_count,
  }));

  const maxTokens = projectStats[0]?.total_tokens || 1;

  return (
    <div className="space-y-5 animate-slide-in">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold">Projects</h2>
        <DateFilter />
      </div>

      <div className="glass-card rounded-xl p-5">
        <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
          <span className="w-1.5 h-1.5 rounded-full bg-accent" />
          Token Usage by Project
        </h3>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={chartData}>
            <XAxis dataKey="name" tick={{ fontSize: 10, fill: "#6b7280" }} axisLine={false} tickLine={false} />
            <YAxis tick={{ fontSize: 10, fill: "#6b7280" }} tickFormatter={formatTokens} axisLine={false} tickLine={false} />
            <Tooltip contentStyle={tooltipStyle} />
            <Bar dataKey="tokens" fill="#22d3ee" radius={[4, 4, 0, 0]} maxBarSize={50} />
          </BarChart>
        </ResponsiveContainer>
      </div>

      <div className="grid gap-3">
        {projectStats.slice(0, 20).map((p, i) => {
          const name = p.project_name;
          const dir = projectDir(p.project_id);
          const pct = Math.min(100, (p.total_tokens / maxTokens) * 100);
          return (
            <div key={p.project_id} className="glass-card rounded-xl p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3 min-w-0">
                  <span className="text-[11px] text-gray-600 w-5 text-right">{i + 1}</span>
                  <Folder className="w-4 h-4 text-accent flex-shrink-0" />
                  <div className="min-w-0">
                    <span className="font-medium text-sm truncate block">{name}</span>
                    <span className="text-[10px] text-gray-600 truncate block max-w-[200px]">{dir}</span>
                  </div>
                </div>
                <div className="flex items-center gap-5 text-sm flex-shrink-0">
                  <div className="text-right">
                    <div className="text-[10px] text-gray-500 uppercase tracking-wider">Sessions</div>
                    <div className="text-xs font-medium">{p.session_count}</div>
                  </div>
                  <div className="text-right">
                    <div className="text-[10px] text-gray-500 uppercase tracking-wider">Tokens</div>
                    <div className="text-xs font-medium text-yellow-accent">{formatTokens(p.total_tokens)}</div>
                  </div>
                  <div className="text-right">
                    <div className="text-[10px] text-gray-500 uppercase tracking-wider">Cost</div>
                    <div className="text-xs font-mono text-green-accent">{formatCost(p.total_cost)}</div>
                  </div>
                </div>
              </div>
              <div className="mt-3 h-1.5 bg-surface-600/50 rounded-full overflow-hidden">
                <div
                  className="h-full rounded-full bg-gradient-to-r from-accent to-accent/40"
                  style={{ width: `${pct}%` }}
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
