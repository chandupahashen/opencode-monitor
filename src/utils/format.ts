export function formatCost(c: number): string {
  if (c >= 1_000_000_000_000) return `$${(c / 1_000_000_000_000).toFixed(2)}T`;
  if (c >= 1_000_000_000) return `$${(c / 1_000_000_000).toFixed(2)}B`;
  if (c >= 1_000_000) return `$${(c / 1_000_000).toFixed(2)}M`;
  if (c >= 1_000) return `$${(c / 1_000).toFixed(2)}K`;
  if (c >= 0.01) return `$${c.toFixed(2)}`;
  return `$${c.toFixed(4)}`;
}

export function formatTokens(n: number): string {
  if (n >= 1_000_000_000_000) return `${(n / 1_000_000_000_000).toFixed(1)}T`;
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}
