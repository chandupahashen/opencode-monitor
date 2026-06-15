export function Skeleton({ className = "" }: { className?: string }) {
  return (
    <div
      className={`rounded-md bg-surface-700 animate-skeleton ${className}`}
    />
  );
}

export function SkeletonCard({ className = "" }: { className?: string }) {
  return (
    <div className={`glass-card rounded-xl p-5 ${className}`}>
      <div className="flex items-center gap-2 mb-4">
        <Skeleton className="w-1.5 h-1.5 rounded-full" />
        <Skeleton className="h-3.5 w-32" />
      </div>
      <Skeleton className="h-[220px] w-full rounded-lg" />
    </div>
  );
}

export function SkeletonTable({ rows = 6 }: { rows?: number }) {
  return (
    <div className="glass-card rounded-xl overflow-hidden">
      <div className="p-4 border-b border-border">
        <div className="flex items-center justify-between">
          <Skeleton className="h-4 w-48" />
          <Skeleton className="h-4 w-32" />
        </div>
      </div>
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 p-3 border-b border-border/50">
          <Skeleton className="h-3 w-16" />
          <Skeleton className="h-3 w-40 flex-1" />
          <Skeleton className="h-3 w-20" />
          <Skeleton className="h-3 w-14" />
          <Skeleton className="h-3 w-14" />
          <Skeleton className="h-3 w-12" />
        </div>
      ))}
    </div>
  );
}

export function SkeletonKPI() {
  return (
    <div className="glass-card rounded-xl p-4">
      <Skeleton className="h-3 w-20 mb-3" />
      <Skeleton className="h-7 w-16" />
    </div>
  );
}

export function AppLoader() {
  return (
    <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-surface-900">
      <div className="flex items-center gap-3 mb-4">
        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-cyan-600 flex items-center justify-center">
          <svg className="w-4 h-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </div>
        <span className="text-sm font-semibold text-gray-300">openCode Monitor</span>
      </div>
      <div className="app-loader" />
      <p className="mt-4 text-xs text-gray-600">Loading data...</p>
    </div>
  );
}
