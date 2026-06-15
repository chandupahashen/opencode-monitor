export function projectName(id: string | null | undefined): string {
  if (!id) return "—";
  const parts = id.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length > 1) return parts[parts.length - 1] ?? id;
  return id;
}

export function projectDir(id: string | null | undefined): string {
  if (!id) return "";
  const parts = id.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 1) return "";
  parts.pop();
  return parts.join("/");
}
