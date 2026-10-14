import { useMemo, useState } from "react";
import { createPortal } from "react-dom";
import { useStore } from "../store/useStore";
import { formatTokens } from "../utils/format";
import { Skeleton } from "./Skeleton";
import {
  aggregateWeeks,
  applyCumulative,
  buildDayColumns,
  columnMonthLabels,
  formatDayShort,
  formatWeekShort,
  levelFor,
} from "../utils/heatmap";

type Mode = "daily" | "weekly" | "cumulative";

const MODES: { id: Mode; label: string }[] = [
  { id: "daily", label: "Daily" },
  { id: "weekly", label: "Weekly" },
  { id: "cumulative", label: "Cumulative" },
];

const EMPTY_COLOR = "#26262f";
const SCALE = ["#4a2c17", "#8a4d1c", "#c97a2e", "#f0a35e", "#fab283"];

function colorFor(tokens: number, max: number): string {
  const lvl = levelFor(tokens, max, SCALE.length);
  return lvl === 0 ? EMPTY_COLOR : SCALE[lvl - 1];
}

interface Tooltip {
  x: number;
  y: number;
  title: string;
  sub: string;
}

export function TokenActivity() {
  const dailyActivity = useStore((s) => s.dailyActivity);
  const isLoading = useStore((s) => s.isLoading);
  const [mode, setMode] = useState<Mode>("daily");
  const [tip, setTip] = useState<Tooltip | null>(null);

  const data = useMemo(() => {
    const byDate = new Map<string, number>();
    for (const d of dailyActivity) byDate.set(d.date, d.total_tokens);
    const columns = buildDayColumns(byDate);
    const labels = columnMonthLabels(columns);
    const total = columns.flat().reduce((s, d) => s + d.tokens, 0);
    const activeDays = columns.flat().filter((d) => d.tokens > 0).length;
    const maxDaily = Math.max(0, ...columns.flat().map((d) => d.tokens));
    const weeks = aggregateWeeks(columns);
    const maxWeekly = Math.max(0, ...weeks.map((w) => w.tokens));
    const cumulative = applyCumulative(columns);
    return { columns, labels, total, activeDays, maxDaily, weeks, maxWeekly, cumulative };
  }, [dailyActivity]);

  if (isLoading && dailyActivity.length === 0) {
    return (
      <div className="glass-card rounded-xl p-5">
        <Skeleton className="h-5 w-32 mb-4" />
        <Skeleton className="h-28 w-full" />
      </div>
    );
  }

  const colCount = data.columns.length || 53;
  const gap = mode === "weekly" ? 4 : 3;
  // Equal fractional columns: the strip always fits the card exactly, no scrolling.
  const gridStyle: React.CSSProperties = {
    display: "grid",
    gridTemplateColumns: `repeat(${colCount}, minmax(0, 1fr))`,
    gap,
  };

  function showTip(e: React.MouseEvent, title: string, sub: string) {
    setTip({ x: e.clientX, y: e.clientY, title, sub });
  }

  return (
    <div className="glass-card rounded-xl p-5" onMouseLeave={() => setTip(null)}>
      <div className="flex items-center justify-between flex-wrap gap-2 mb-1">
        <h3 className="text-sm font-medium text-gray-200">Token activity</h3>
        <div className="flex items-center gap-1 text-xs">
          {MODES.map((m) => (
            <button
              key={m.id}
              onClick={() => setMode(m.id)}
              className={`px-2.5 py-1 rounded-md transition-colors font-medium ${
                mode === m.id
                  ? "text-gray-100 bg-surface-600/80"
                  : "text-gray-500 hover:text-gray-300"
              }`}
            >
              {m.label}
            </button>
          ))}
        </div>
      </div>
      <p className="text-[11px] text-gray-500 mb-4">
        {formatTokens(data.total)} tokens · {data.activeDays} active days · last 12 months
      </p>

      <div className="pb-1">
        {mode === "weekly" ? (
          <div>
            <div style={gridStyle} role="img" aria-label="Weekly token activity">
              {data.weeks.map((w) => (
                <div
                  key={w.startTs}
                  onMouseEnter={(e) =>
                    showTip(e, formatWeekShort(w.startTs, w.endTs), `${formatTokens(w.tokens)} tokens`)
                  }
                  className="rounded-[3px] aspect-square w-full transition-transform hover:scale-110"
                  style={{ backgroundColor: colorFor(w.tokens, data.maxWeekly) }}
                />
              ))}
            </div>
            <div className="grid mt-1.5" style={gridStyle}>
              {data.labels.map((label, i) => (
                <div
                  key={i}
                  className={`text-[10px] leading-none text-gray-500 whitespace-nowrap pt-0.5 ${
                    i === data.labels.length - 1 ? "text-right" : ""
                  }`}
                >
                  {label ?? ""}
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div>
            <div
              style={gridStyle}
              role="img"
              aria-label={`${mode === "cumulative" ? "Cumulative" : "Daily"} token activity`}
            >
              {(mode === "cumulative" ? data.cumulative : data.columns).map((days, wi) => (
                <div key={wi} className="grid" style={{ gap, alignContent: "start" }}>
                  {days.map((d) => {
                    const max = mode === "cumulative" ? data.total : data.maxDaily;
                    return (
                      <div
                        key={d.date}
                        onMouseEnter={(e) =>
                          showTip(
                            e,
                            formatDayShort(d.ts),
                            `${formatTokens(d.tokens)}${mode === "cumulative" ? " total" : ""} tokens`,
                          )
                        }
                        className="rounded-[3px] aspect-square w-full transition-transform hover:scale-110"
                        style={{ backgroundColor: colorFor(d.tokens, max) }}
                      />
                    );
                  })}
                </div>
              ))}
            </div>
            <div className="grid mt-1.5" style={gridStyle}>
              {data.labels.map((label, i) => (
                <div
                  key={i}
                  className={`text-[10px] leading-none text-gray-500 whitespace-nowrap pt-0.5 ${
                    i === data.labels.length - 1 ? "text-right" : ""
                  }`}
                >
                  {label ?? ""}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      <div className="flex items-center justify-end gap-1.5 mt-3 text-[10px] text-gray-500">
        <span>Less</span>
        <div
          className="rounded-[3px]"
          style={{ width: 11, height: 11, backgroundColor: EMPTY_COLOR }}
        />
        {SCALE.map((c) => (
          <div key={c} className="rounded-[3px]" style={{ width: 11, height: 11, backgroundColor: c }} />
        ))}
        <span>More</span>
      </div>

      {tip &&
        createPortal(
          <div
            className="fixed z-[100] pointer-events-none px-3 py-2 rounded-xl whitespace-nowrap"
            style={{
              // Clamp inside the viewport; flip below the cursor near the top edge.
              left: Math.min(Math.max(tip.x, 84), window.innerWidth - 84),
              top: tip.y < 96 ? tip.y + 18 : tip.y - 12,
              transform: tip.y < 96 ? "translate(-50%, 0)" : "translate(-50%, -100%)",
              background: "#2b2b36",
              border: "1px solid #3a3a48",
              boxShadow: "0 4px 20px rgba(0,0,0,0.5)",
            }}
          >
            <div className="text-xs text-gray-200 font-medium leading-tight">{tip.title}</div>
            <div className="text-xs text-gray-400 leading-tight mt-0.5">{tip.sub}</div>
          </div>,
          document.body,
        )}
    </div>
  );
}
