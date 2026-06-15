export interface DecodedModel {
  id: string;
  provider: string;
  variant: string | null;
  discount: number | null;
  display: string;
  short: string;
  color: string;
}

export function decodeModel(raw: string | null): DecodedModel {
  if (!raw) {
    return { id: "—", provider: "", variant: null, discount: null, display: "—", short: "—", color: "text-gray-500" };
  }

  let id = raw;
  let provider = "";
  let variant: string | null = null;
  let discount: number | null = null;

  const pctMatch = raw.match(/^(.+?)\s+(\d+)%$/);
  if (pctMatch) {
    id = pctMatch[1].trim();
    discount = parseInt(pctMatch[2], 10);
  }

  try {
    const parsed = JSON.parse(id);
    if (parsed && typeof parsed === "object") {
      id = parsed.id ?? id;
      provider = parsed.providerID ?? "";
      variant = parsed.variant ?? null;
    }
  } catch {
    const slashIdx = id.indexOf("/");
    if (slashIdx > 0) {
      provider = id.slice(0, slashIdx);
      id = id.slice(slashIdx + 1);
    }
  }

  const short = id;
  let display = id;
  if (provider) display = `${provider}/${id}`;
  if (variant) display += ` (${variant})`;
  if (discount !== null) display += ` ${discount}%`;

  const color = getModelColor(id);

  return { id, provider, variant, discount, display, short, color };
}

function getModelColor(model: string): string {
  const m = model.toLowerCase();
  if (m.includes("gpt")) return "text-green-accent";
  if (m.includes("claude")) return "text-orange-accent";
  if (m.includes("deepseek")) return "text-accent";
  if (m.includes("gemini")) return "text-blue-400";
  if (m.includes("llama")) return "text-purple-accent";
  if (m.includes("o1") || m.includes("o3")) return "text-yellow-accent";
  return "text-gray-300";
}
