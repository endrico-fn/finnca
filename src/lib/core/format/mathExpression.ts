import type { SupportedCurrency } from './currency';

export function evaluateFinancialExpression(
  expr: string,
  currency: SupportedCurrency = 'IDR'
): number | null {
  if (!expr || typeof expr !== 'string') return null;

  let s = expr.trim().toLowerCase().replace(/^(rp|usd|\$)\s*/, '');
  if (!s) return null;

  s = s.replace(/(\d+),(\d+)\s*(k|rb|m|jt)\b/g, '$1.$2$3');

  if (currency.toUpperCase() === 'IDR') {
    while (/(\d)[.,](\d{3})(?=[.,+\-*/()\s]|$)(?![0-9]|[a-z])/i.test(s)) {
      s = s.replace(/(\d)[.,](\d{3})(?=[.,+\-*/()\s]|$)(?![0-9]|[a-z])/gi, '$1$2');
    }
  } else {
    while (/(\d),(\d{3})(?=[,.\D]|$)(?![0-9]|[a-z])/i.test(s)) {
      s = s.replace(/(\d),(\d{3})(?=[,.\D]|$)(?![0-9]|[a-z])/gi, '$1$2');
    }
    while (/(\d)\.(\d{3})(?=\.\d{3})/i.test(s)) {
      s = s.replace(/(\d)\.(\d{3})(?=\.\d{3})/gi, '$1$2');
    }
  }

  const tokenRegex = /(\d+(?:\.\d+)?(?:k|rb|m|jt)?|[+\-*/()])/g;
  const tokens: string[] = [];
  let match: RegExpExecArray | null;
  let lastIndex = 0;

  while ((match = tokenRegex.exec(s)) !== null) {
    const skipped = s.slice(lastIndex, match.index).trim();
    if (skipped) return null;
    tokens.push(match[0]);
    lastIndex = tokenRegex.lastIndex;
  }

  const remainder = s.slice(lastIndex).trim();
  if (remainder) return null;
  if (tokens.length === 0) return null;

  let cursor = 0;

  function parseNumber(token: string): number | null {
    if (token.endsWith('k') || token.endsWith('rb')) {
      const raw = token.slice(0, token.endsWith('rb') ? -2 : -1);
      const num = parseFloat(raw);
      return isNaN(num) ? null : num * 1000;
    }
    if (token.endsWith('m') || token.endsWith('jt')) {
      const raw = token.slice(0, token.endsWith('jt') ? -2 : -1);
      const num = parseFloat(raw);
      return isNaN(num) ? null : num * 1000000;
    }
    const num = parseFloat(token);
    return isNaN(num) ? null : num;
  }

  function parseFactor(): number | null {
    if (cursor >= tokens.length) return null;
    const token = tokens[cursor];

    if (token === '+') {
      cursor++;
      return parseFactor();
    }
    if (token === '-') {
      cursor++;
      const val = parseFactor();
      return val === null ? null : -val;
    }
    if (token === '(') {
      cursor++;
      const val = parseExpression();
      if (val === null) return null;
      if (cursor >= tokens.length || tokens[cursor] !== ')') return null;
      cursor++;
      return val;
    }

    const num = parseNumber(token);
    if (num === null) return null;
    cursor++;
    return num;
  }

  function parseTerm(): number | null {
    let left = parseFactor();
    if (left === null) return null;

    while (cursor < tokens.length) {
      const op = tokens[cursor];
      if (op === '*' || op === '/') {
        cursor++;
        const right = parseFactor();
        if (right === null) return null;
        if (op === '*') {
          left = left * right;
        } else {
          if (right === 0) return null;
          left = left / right;
        }
      } else {
        break;
      }
    }

    return left;
  }

  function parseExpression(): number | null {
    let left = parseTerm();
    if (left === null) return null;

    while (cursor < tokens.length) {
      const op = tokens[cursor];
      if (op === '+' || op === '-') {
        cursor++;
        const right = parseTerm();
        if (right === null) return null;
        if (op === '+') {
          left = left + right;
        } else {
          left = left - right;
        }
      } else {
        break;
      }
    }

    return left;
  }

  const result = parseExpression();
  if (result === null || cursor < tokens.length || !Number.isFinite(result)) {
    return null;
  }

  return result;
}

export function hasMathExpression(str: string): boolean {
  if (!str) return false;
  return /[+\-*/()]|[0-9]+[.,]?[0-9]*(k|rb|m|jt)\b/i.test(str.trim());
}
