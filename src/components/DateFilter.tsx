import { useState } from "react";
import { useStore } from "../store/useStore";
import { Calendar, X } from "lucide-react";

function toDateStr(d: Date): string {
  return d.toISOString().slice(0, 10);
}

const presets = [
  { value: "all", label: "All time", days: null },
  { value: "today", label: "Today", days: 0 },
  { value: "3d", label: "Last 3 days", days: 3 },
  { value: "week", label: "Last week", days: 7 },
  { value: "month", label: "Last month", days: 30 },
  { value: "year", label: "Last year", days: 365 },
  { value: "custom", label: "Custom range", days: null },
] as const;

type PresetValue = (typeof presets)[number]["value"];

export function DateFilter() {
  const dateFrom = useStore((s) => s.dateFrom);
  const dateTo = useStore((s) => s.dateTo);
  const setDateFrom = useStore((s) => s.setDateFrom);
  const setDateTo = useStore((s) => s.setDateTo);
  const [activePreset, setActivePreset] = useState<PresetValue>("all");

  const isCustom = activePreset === "custom";
  const hasFilter = dateFrom || dateTo;

  function applyPreset(value: PresetValue) {
    if (value === "all") {
      setDateFrom("");
      setDateTo("");
      setActivePreset("all");
      return;
    }
    if (value === "custom") {
      setActivePreset("custom");
      return;
    }
    const preset = presets.find((p) => p.value === value);
    if (!preset || preset.days === null) return;
    const today = new Date();
    const to = toDateStr(today);
    if (preset.days === 0) {
      setDateFrom(to);
    } else {
      const from = new Date(today);
      from.setDate(from.getDate() - preset.days);
      setDateFrom(toDateStr(from));
    }
    setDateTo(to);
    setActivePreset(value);
  }

  function clear() {
    setDateFrom("");
    setDateTo("");
    setActivePreset("all");
  }

  function handleFromChange(e: React.ChangeEvent<HTMLInputElement>) {
    setDateFrom(e.target.value);
    if (e.target.value || dateTo) setActivePreset("custom");
  }

  function handleToChange(e: React.ChangeEvent<HTMLInputElement>) {
    setDateTo(e.target.value);
    if (e.target.value || dateFrom) setActivePreset("custom");
  }

  const activeLabel = presets.find((p) => p.value === activePreset)?.label ?? "All time";

  return (
    <div className="flex items-center gap-2">
      <div className="relative">
        <select
          value={activePreset}
          onChange={(e) => applyPreset(e.target.value as PresetValue)}
          className="px-2.5 py-1.5 pr-7 bg-surface-800 border border-border rounded text-xs focus:outline-none focus:border-accent/50 text-gray-300 appearance-none cursor-pointer"
        >
          {presets.map((p) => (
            <option key={p.value} value={p.value}>
              {p.label}
            </option>
          ))}
        </select>
        <Calendar className="absolute right-2 top-1/2 -translate-y-1/2 w-3 h-3 text-gray-500 pointer-events-none" />
      </div>

      {isCustom && (
        <>
          <span className="text-gray-600 text-xs">—</span>
          <input
            type="date"
            value={dateFrom}
            onChange={handleFromChange}
            className="px-2 py-1.5 bg-surface-800 border border-border rounded text-xs focus:outline-none focus:border-accent/50 text-gray-300 w-[130px]"
            title="From date"
          />
          <span className="text-gray-600 text-xs">to</span>
          <input
            type="date"
            value={dateTo}
            onChange={handleToChange}
            className="px-2 py-1.5 bg-surface-800 border border-border rounded text-xs focus:outline-none focus:border-accent/50 text-gray-300 w-[130px]"
            title="To date"
          />
        </>
      )}

      {!isCustom && activePreset !== "all" && (
        <span className="text-[10px] text-accent font-medium px-1">
          {activeLabel}
        </span>
      )}

      {hasFilter && (
        <button
          onClick={clear}
          className="p-1 hover:bg-surface-700 rounded transition-colors text-gray-500 hover:text-gray-300"
          title="Clear filters"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      )}
    </div>
  );
}
