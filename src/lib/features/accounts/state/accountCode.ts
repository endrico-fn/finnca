import type { Account } from '$lib/core/ipc/bindings';

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

export function parentPrefixOf(code: string): string {
  const digits = code.replace(/[^0-9]/g, '');
  if (digits.length <= 2) return digits;
  if (digits.length === 4) return digits.slice(0, 3);
  return digits.slice(0, 2);
}

export function isOutsideParentPrefix(code: string, parentCode: string | null): boolean {
  if (!parentCode) return false;
  const c = code.replace(/[^0-9]/g, '');
  const p = parentCode.replace(/[^0-9]/g, '');
  if (!c || !p) return false;
  return !c.startsWith(p.slice(0, Math.min(3, p.length)));
}

export function siblingCodes(accounts: Account[], parentId: string | null): string[] {
  return accounts.filter((a) => (a.parent_id ?? null) === (parentId ?? null)).map((a) => a.code);
}
