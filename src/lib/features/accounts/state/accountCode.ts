export function suggestAccountCode(parentCode: string | null, taken: Set<string>): string {
  if (!parentCode) return '';
  const digits = parentCode.replace(/[^0-9]/g, '');
  if (digits.length < 2) return '';
  const head = digits.slice(0, 2);
  for (let tail = 10; tail <= 99; tail += 1) {
    const code = `${head}${String(tail)}`;
    if (!taken.has(code)) return code;
  }
  let n = Number(`${head}00`) + 1;
  while (taken.has(String(n))) n += 1;
  return String(n);
}

export function isOutsideParentPrefix(code: string, parentCode: string | null): boolean {
  if (!parentCode) return false;
  const c = code.replace(/[^0-9]/g, '');
  const p = parentCode.replace(/[^0-9]/g, '');
  if (!c || !p) return false;
  return !c.startsWith(p.slice(0, Math.min(3, p.length)));
}
