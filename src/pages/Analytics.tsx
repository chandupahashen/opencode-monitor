import { useStore } from "../store/useStore";
import {
  BarChart, Bar, LineChart, Line, AreaChart, Area,
  XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid, Legend,
} from "recharts";
import { decodeModel } from "../utils/model";
import { DateFilter } from "../components/DateFilter";
import { SkeletonCard, Skeleton } from "../components/Skeleton";

function formatTokens(n: number) {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

function formatCost(c: number) {
  return `$${c.toFixed(4)}`;
}

const tooltipStyle = { background: "#1a1a25", border: "1px solid #2a2a3e", borderRadius: 8, fontSize: 12, boxShadow: "0 4px 20px rgba(0,0,0,0.4)" };

export function Analytics() {
  const tokenTrends = useStore((s) => s.tokenTrends);
  const modelUsage = useStore((s) => s.modelUsage);
  const costBreakdown = useStore((s) => s.costBreakdown);
  const isLoading = useStore((s) => s.isLoading);

  if (isLoading && tokenTrends.length === 0) {
    return (
      <div className="space-y-5">
        <Skeleton className="h-5 w-24" />
        <SkeletonCard />
        <div className="grid grid-cols-2 gap-5">
          <SkeletonCard />
          <SkeletonCard />
        </div>
        <div className="grid grid-cols-2 gap-5">
          <SkeletonCard />
          <SkeletonCard />
        </div>
      </div>
    );
  }

  const stackedTrend = tokenTrends.map((t) => ({
    date: t.date,
    input: t.tokens_input,
    output: t.tokens_output,
    reasoning: t.tokens_reasoning,
    cache_read: t.tokens_cache_read,
    cache_write: t.tokens_cache_write,
    cost: t.cost,
  }));

  return (
    <div className="space-y-5 animate-slide-in">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold">Analytics</h2>
        <DateFilter />
      </div>

      <div className="glass-card rounded-xl p-5">
        <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
          <span className="w-1.5 h-1.5 rounded-full bg-accent" />
          Token Trends
        </h3>
        <ResponsiveContainer width="100%" height={300}>
          <AreaChart data={stackedTrend}>
            <CartesianGrid strokeDasharray="3 3" stroke="#2a2a3e" strokeOpacity={0.5} />
            <XAxis dataKey="date" tick={{ fontSize: 10, fill: "#6b7280" }} axisLine={false} tickLine={false} />
            <YAxis tick={{ fontSize: 10, fill: "#6b7280" }} tickFormatter={formatTokens} axisLine={false} tickLine={false} />
            <Tooltip contentStyle={tooltipStyle} />
            <Legend wrapperStyle={{ fontSize: 11, paddingTop: 8 }} />
            <Area type="monotone" dataKey="input" stackId="1" stroke="#22d3ee" fill="#22d3ee" fillOpacity={0.4} strokeWidth={1.5} />
            <Area type="monotone" dataKey="output" stackId="1" stroke="#34d399" fill="#34d399" fillOpacity={0.4} strokeWidth={1.5} />
            <Area type="monotone" dataKey="reasoning" stackId="1" stroke="#fbbf24" fill="#fbbf24" fillOpacity={0.4} strokeWidth={1.5} />
            <Area type="monotone" dataKey="cache_read" stackId="1" stroke="#a78bfa" fill="#a78bfa" fillOpacity={0.4} strokeWidth={1.5} />
            <Area type="monotone" dataKey="cache_write" stackId="1" stroke="#f472b6" fill="#f472b6" fillOpacity={0.4} strokeWidth={1.5} />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      <div className="grid grid-cols-2 gap-5">
        <div className="glass-card rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-green-accent" />
            Cost Over Time
          </h3>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={stackedTrend}>
              <CartesianGrid strokeDasharray="3 3" stroke="#2a2a3e" strokeOpacity={0.5} />
              <XAxis dataKey="date" tick={{ fontSize: 10, fill: "#6b7280" }} axisLine={false} tickLine={false} />
              <YAxis tick={{ fontSize: 10, fill: "#6b7280" }} tickFormatter={(v: number) => `$${v.toFixed(2)}`} axisLine={false} tickLine={false} />
              <Tooltip contentStyle={tooltipStyle} />
              <Line type="monotone" dataKey="cost" stroke="#34d399" strokeWidth={2.5} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        <div className="glass-card rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-accent" />
            Model Comparison
          </h3>
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={modelUsage.slice(0, 10)}>
              <CartesianGrid strokeDasharray="3 3" stroke="#2a2a3e" strokeOpacity={0.5} />
              <XAxis dataKey="model" tick={{ fontSize: 9, fill: "#6b7280" }} tickFormatter={(v: string) => decodeModel(v).short} axisLine={false} tickLine={false} />
              <YAxis tick={{ fontSize: 10, fill: "#6b7280" }} tickFormatter={formatTokens} axisLine={false} tickLine={false} />
              <Tooltip contentStyle={tooltipStyle} />
              <Bar dataKey="total_tokens" fill="#22d3ee" radius={[4, 4, 0, 0]} maxBarSize={40} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-5">
        <div className="glass-card rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-green-accent" />
            Cost by Model
          </h3>
          <div className="space-y-1.5">
            {(costBreakdown?.by_model ?? []).slice(0, 10).map((m, i) => (
              <div key={m.model} className="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-surface-700/30 transition-colors">
                <div className="flex items-center gap-2 min-w-0">
                  <span className="text-[11px] text-gray-600 w-4 text-right">{i + 1}</span>
                  <span className="text-sm text-gray-300 truncate max-w-[180px]">{decodeModel(m.model).short}</span>
                </div>
                <div className="flex items-center gap-3 flex-shrink-0">
                  <span className="text-[11px] text-gray-500">{formatTokens(m.total_tokens)} tok</span>
                  <span className="text-green-accent font-mono text-xs w-16 text-right">{formatCost(m.total_cost)}</span>
                </div>
              </div>
            ))}
            {(costBreakdown?.by_model ?? []).length === 0 && (
              <div className="text-center py-8 text-gray-600 text-sm">No model data</div>
            )}
          </div>
        </div>

        <div className="glass-card rounded-xl p-5">
          <h3 className="text-sm font-medium text-gray-300 mb-4 flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-purple-accent" />
            Cost by Project
          </h3>
          <div className="space-y-1.5 max-h-[280px] overflow-y-auto">
            {(costBreakdown?.by_project ?? []).slice(0, 15).map((p, i) => (
              <div key={p.project_id} className="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-surface-700/30 transition-colors">
                <div className="flex items-center gap-2 min-w-0">
                  <span className="text-[11px] text-gray-600 w-4 text-right">{i + 1}</span>
                  <span className="text-sm text-gray-300 truncate max-w-[160px]">{p.project_name}</span>
                </div>
                <div className="flex items-center gap-3 flex-shrink-0">
                  <span className="text-[11px] text-gray-500">{p.session_count} ses</span>
                  <span className="text-green-accent font-mono text-xs w-16 text-right">{formatCost(p.total_cost)}</span>
                </div>
              </div>
            ))}
            {(costBreakdown?.by_project ?? []).length === 0 && (
              <div className="text-center py-8 text-gray-600 text-sm">No project data</div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
