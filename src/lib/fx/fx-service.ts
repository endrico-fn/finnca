/** Shared FX util — dedup fetch live USD/IDR rate, Abort 3.5s, no key, format helper */
const FX_ENDPOINT = 'https://open.er-api.com/v6/latest/USD';
const FX_ABORT_MS = 3500;

export async function fetchLiveFxRate(): Promise<number | null> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), FX_ABORT_MS);
  try {
    const res = await fetch(FX_ENDPOINT, { signal: controller.signal });
    if (!res.ok) return null;
    const data = await res.json();
    const rate = Math.round(data?.rates?.IDR ?? 0);
    return rate > 1000 ? rate : null;
  } catch {
    return null;
  } finally {
    clearTimeout(timeoutId);
  }
}
