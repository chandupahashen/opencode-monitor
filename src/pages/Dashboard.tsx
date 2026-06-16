import { useStore } from "../store/useStore";
import { Activity, Cpu, DollarSign, Zap, BookOpen, Briefcase } from "lucide-react";
import { AreaChart, Area, BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import { useMemo } from "react";
import { decodeModel } from "../utils/model";
import { DateFilter } from "../components/DateFilter";
import { SkeletonCard, SkeletonKPI, Skeleton } from "../components/Skeleton";

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

const COLORS = ["#22d3ee", "#fbbf24", "#34d399", "#a78bfa", "#fb7185", "#f472b6", "#2dd4bf", "#fb923c"];

const kpiConfig = [
  { label: "Total Sessions", key: "total_sessions" as const, icon: Activity, color: "text-accent", bg: "from-accent/10 to-transparent" },
  { label: "Total Tokens", key: "tokens" as const, icon: Cpu, color: "text-yellow-accent", bg: "from-yellow-accent/10 to-transparent" },
  { label: "Total Cost", key: "cost" as const, icon: DollarSign, color: "text-green-accent", bg: "from-green-accent/10 to-transparent" },
  { label: "Active", key: "active_sessions" as const, icon: Zap, color: "text-purple-accent", bg: "from-purple-accent/10 to-transparent" },
  { label: "Projects", key: "total_projects" as const, icon: Briefcase, color: "text-pink-accent", bg: "from-pink-accent/10 to-transparent" },
  { label: "Cache Tokens", key: "cache" as const, icon: BookOpen, color: "text-orange-accent", bg: "from-orange-accent/10 to-transparent" },
];

export function Dashboard() {
  const overview = useStore((s) => s.overview);
  const tokenTrends = useStore((s) => s.tokenTrends);
  const modelUsage = useStore((s) => s.modelUsage);

  const totalTokens = useMemo(() => {
    if (!overview) return 0;
    return overview.total_tokens_input + overview.total_tokens_output + overview.total_tokens_reasoning;
  }, [overview]);

  const cacheTokens = useMemo(() => {
    if (!overview) return 0;
    return overview.total_tokens_cache_read + overview.total_tokens_cache_write;
  }, [overview]);

  if (!overview) {
    return (
      <div className="space-y-6 animate-skeleton">
        <Skeleton className="h-5 w-32" />
        <div className="grid grid-cols-6 gap-3">
          {Array.from({ length: 6 }).map((_, i) => <SkeletonKPI key={i} />)}
        </div>
        <div className="grid grid-cols-2 gap-5">
          <SkeletonCard />
          <SkeletonCard />
        </div>
      </div>
    );
  }

  const kpiValues: Record<string, string | number> = {
    total_sessions: String(overview.total_sessions),
    tokens: formatTokens(totalTokens),
    cost: formatCost(overview.total_cost),
    active_sessions: String(overview.active_sessions),
    total_projects: String(overview.total_projects),
    cache: formatTokens(cacheTokens),
  };

  const trendData = tokenTrends.map((t) => ({
    date: t.date,
    total: t.tokens_input + t.tokens_output + t.tokens_reasoning,
  }));

  const tooltipStyle = { background: "#1a1a25", border: "1px solid #2a2a3e", borderRadius: 8, fontSize: 12, boxShadow: "0 4px 20px rgba(0,0,0,0.4)" };

  return (
    <div className="space-y-6 animate-slide-in">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold">Dashboard</h2>
        <DateFilter />
      </div>
      <div className="grid grid-cols-6 gap-3">
        {kpiConfig.map((kpi) => {
          const Icon = kpi.icon;
          const val = kpiValues[kpi.key];
          return (
            <div key={kpi.key} className="glass-card rounded-xl p-4 relative overflow-hidden">
              <div className={`absolute inset-0 bg-gradient-to-br ${kpi.bg} opacity-50`} />
              <div className="relative z-10">
                <div className="flex items-center gap-1.5 text-[11px] text-gray-500 mb-2 uppercase tracking-wider font-medium">
                  <Icon className={`w-3.5 h-3.5 ${kpi.color}`} />
                  {kpi.label}
                </div>
                <div className={`text-2xl font-bold ${kpi.color}`}>{val}</div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="grid grid-cols-2 gap-5">
        <div className="glass-card rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-accent" />
            Token Usage <span className="text-gray-500 font-normal">(last 30d)</span>
          </h3>
          <ResponsiveContainer width="100%" height={220}>
            <AreaChart data={trendData}>
              <defs>
                <linearGradient id="inputGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#22d3ee" stopOpacity={0.25} />
                  <stop offset="100%" stopColor="#22d3ee" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis dataKey="date" tick={{ fontSize: 10, fill: "#6b7280" }} axisLine={false} tickLine={false} />
              <YAxis tick={{ fontSize: 10, fill: "#6b7280" }} tickFormatter={formatTokens} axisLine={false} tickLine={false} />
              <Tooltip contentStyle={tooltipStyle} formatter={(value: unknown) => [formatTokens(Number(value)), "Total"]} />
              <Area type="monotone" dataKey="total" stroke="#22d3ee" fill="url(#inputGrad)" strokeWidth={2.5} dot={false} />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        <div className="glass-card rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-yellow-accent" />
            Model Distribution
          </h3>
          {modelUsage.length > 0 ? (
            <div className="space-y-2">
              <ResponsiveContainer width="100%" height={200}>
                <BarChart data={modelUsage.slice(0, 10)} layout="vertical" margin={{ left: 0, right: 20, top: 0, bottom: 0 }}>
                  <XAxis type="number" tick={{ fontSize: 10, fill: "#6b7280" }} tickFormatter={formatTokens} axisLine={false} tickLine={false} />
                  <YAxis type="category" dataKey="model" tick={{ fontSize: 10, fill: "#6b7280" }} tickFormatter={(v: string) => decodeModel(v).short} axisLine={false} tickLine={false} width={80} />
                  <Tooltip contentStyle={tooltipStyle} formatter={(value) => formatTokens(Number(value))} />
                  <Bar dataKey="total_tokens" radius={[0, 4, 4, 0]} maxBarSize={16}>
                    {modelUsage.slice(0, 10).map((entry, i) => (
                      <Cell key={entry.model} fill={COLORS[i % COLORS.length]} />
                    ))}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
              <div className="flex flex-wrap gap-x-4 gap-y-1 pt-1">
                {modelUsage.slice(0, 10).map((m, i) => {
                  const dm = decodeModel(m.model);
                  return (
                    <span key={m.model} className="flex items-center gap-1.5 text-[10px] text-gray-400">
                      <span className="w-2 h-2 rounded-full" style={{ backgroundColor: COLORS[i % COLORS.length] }} />
                      <span className={dm.color}>{dm.short}</span>
                    </span>
                  );
                })}
              </div>
            </div>
          ) : (
            <div className="flex items-center justify-center h-[220px] text-gray-600 text-sm">No model data</div>
          )}
        </div>
      </div>
    </div>
  );
}
