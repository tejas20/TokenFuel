export function countdown(reset: string | null, now = Date.now()): string {
  if (!reset) return "Reset not reported";
  const seconds = Math.ceil((Date.parse(reset) - now) / 1000);
  if (!Number.isFinite(seconds)) return "Reset not reported";
  if (seconds <= 0) return "Reset due · refresh";
  const hours = Math.floor(seconds / 3600);
  return hours >= 24
    ? `Resets in ${Math.floor(hours / 24)}d ${hours % 24}h`
    : `Resets in ${hours}h ${Math.floor((seconds % 3600) / 60)}m`;
}
export function percent(value: number | null): string {
  return value === null ? "—" : `${Math.round(value)}%`;
}
