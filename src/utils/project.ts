export function projectName(id: string | null | undefined, dir?: string | null | undefined): string {
  if (!id) return "—";
  const normalized = id.replace(/\\/g, "/");
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length > 1) return parts[parts.length - 1] ?? id;
  if (dir) {
    const dn = dir.replace(/\\/g, "/");
    const dp = dn.split("/").filter(Boolean);
    if (dp.length > 0) return dp[dp.length - 1];
  }
  const v = id.trim();
  if (v.length <= 16) return v;
  return v.slice(0, 12) + "…";
}

export function projectDir(id: string | null | undefined): string {
  if (!id) return "";
  const parts = id.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 1) return "";
  parts.pop();
  return parts.join("/");
}
