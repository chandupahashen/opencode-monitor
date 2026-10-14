/** Pure helpers for the Token activity heatmap (no imports so they stay unit-testable). */

export interface DayCell {
  /** UTC midnight timestamp. */
  ts: number;
  /** YYYY-MM-DD key. */
  date: string;
  tokens: number;
}

export interface WeekCell {
  startTs: number;
  endTs: number;
  tokens: number;
}

const DAY_MS = 86_400_000;

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

export function parseDayUtc(date: string): number {
  const [y, m, d] = date.split("-").map(Number);
  return Date.UTC(y, m - 1, d);
}

export function toDateKey(ts: number): string {
  const d = new Date(ts);
  const y = d.getUTCFullYear();
  const m = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function startOfUtcDay(ts: number): number {
  const d = new Date(ts);
  return Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate());
}

export function monthShort(ts: number): string {
  return MONTHS[new Date(ts).getUTCMonth()];
}

export function formatDayLabel(ts: number): string {
  const d = new Date(ts);
  return `${monthShort(ts)} ${d.getUTCDate()}, ${d.getUTCFullYear()}`;
}

/** Short day label for tooltips: "Sep 27". */
export function formatDayShort(ts: number): string {
  const d = new Date(ts);
  return `${monthShort(ts)} ${d.getUTCDate()}`;
}

export function formatWeekLabel(startTs: number, endTs: number): string {
  const s = new Date(startTs);
  const e = new Date(endTs);
  const year = e.getUTCFullYear();
  if (s.getUTCMonth() === e.getUTCMonth()) {
    return `${monthShort(startTs)} ${s.getUTCDate()} – ${e.getUTCDate()}, ${year}`;
  }
  return `${monthShort(startTs)} ${s.getUTCDate()} – ${monthShort(endTs)} ${e.getUTCDate()}, ${year}`;
}

/** Short week label for tooltips: "Sep 21 – 27" (year omitted, like day tooltips). */
export function formatWeekShort(startTs: number, endTs: number): string {
  const s = new Date(startTs);
  const e = new Date(endTs);
  if (s.getUTCMonth() === e.getUTCMonth()) {
    return `${monthShort(startTs)} ${s.getUTCDate()} – ${e.getUTCDate()}`;
  }
  return `${monthShort(startTs)} ${s.getUTCDate()} – ${monthShort(endTs)} ${e.getUTCDate()}`;
}

/**
 * Build Sunday-first week columns covering the last 53 weeks ending today (UTC).
 * The final column is partial (ends today) — no future cells are emitted.
 */
export function buildDayColumns(tokenByDate: Map<string, number>, endTs?: number): DayCell[][] {
  const end = startOfUtcDay(endTs ?? Date.now());
  const endWeekday = new Date(end).getUTCDay(); // 0 = Sunday
  const start = end - (52 * 7 + endWeekday) * DAY_MS; // Sunday 52 weeks back
  const cols: DayCell[][] = [];
  for (let w = 0; w < 53; w++) {
    const days: DayCell[] = [];
    for (let i = 0; i < 7; i++) {
      const ts = start + (w * 7 + i) * DAY_MS;
      if (ts > end) break;
      const date = toDateKey(ts);
      days.push({ ts, date, tokens: tokenByDate.get(date) ?? 0 });
    }
    if (days.length > 0) cols.push(days);
  }
  return cols;
}

/** Month label per column, shown when the column's starting month changes. */
export function columnMonthLabels(columns: DayCell[][]): (string | null)[] {
  let prev = -1;
  return columns.map((col) => {
    const m = new Date(col[0].ts).getUTCMonth();
    if (m !== prev) {
      prev = m;
      return MONTHS[m];
    }
    return null;
  });
}

/** Map a value to an intensity level 0..steps (sqrt scale so small values stay visible). */
export function levelFor(value: number, max: number, steps = 5): number {
  if (value <= 0 || max <= 0) return 0;
  const t = Math.sqrt(Math.min(1, value / max));
  return Math.min(steps, Math.floor(t * steps) + 1);
}

export function aggregateWeeks(columns: DayCell[][]): WeekCell[] {
  return columns.map((days) => ({
    startTs: days[0].ts,
    endTs: days[days.length - 1].ts,
    tokens: days.reduce((s, d) => s + d.tokens, 0),
  }));
}

/** Replace each day's tokens with the running total up to that day. */
export function applyCumulative(columns: DayCell[][]): DayCell[][] {
  let run = 0;
  return columns.map((col) =>
    col.map((d) => {
      run += d.tokens;
      return { ...d, tokens: run };
    }),
  );
}
