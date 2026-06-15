import { useEffect, useRef } from "react";
import { useStore } from "../store/useStore";
import { useDb } from "./useDb";

export function useAutoRefresh(enabled: boolean) {
  const interval = useStore((s) => s.refreshInterval);
  const { refreshAll } = useDb();
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    if (enabled && interval > 0) {
      timerRef.current = setInterval(refreshAll, interval * 1000);
    }
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [enabled, interval, refreshAll]);
}
