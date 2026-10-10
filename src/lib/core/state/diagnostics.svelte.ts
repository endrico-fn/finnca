export interface ClientCrashReport {
  timestamp: string;
  type: 'unhandledrejection' | 'error' | 'manual';
  message: string;
  stack?: string;
}

const MAX_STORED_CRASHES = 10;
const STORAGE_KEY = 'finnca_client_crashes';

class DiagnosticsStore {
  crashes = $state<ClientCrashReport[]>([]);

  constructor() {
    this.loadFromStorage();
  }

  private loadFromStorage() {
    try {
      if (typeof window !== 'undefined' && window.sessionStorage) {
        const raw = window.sessionStorage.getItem(STORAGE_KEY);
        if (raw) {
          this.crashes = JSON.parse(raw);
        }
      }
    } catch {
      // safe fallback
    }
  }

  private persist() {
    try {
      if (typeof window !== 'undefined' && window.sessionStorage) {
        window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(this.crashes));
      }
    } catch {
      // safe fallback
    }
  }

  captureError(type: 'unhandledrejection' | 'error' | 'manual', message: string, stack?: string) {
    const sanitizedMsg = this.sanitize(message);
    const sanitizedStack = stack ? this.sanitize(stack) : undefined;
    const report: ClientCrashReport = {
      timestamp: new Date().toISOString(),
      type,
      message: sanitizedMsg,
      stack: sanitizedStack,
    };
    this.crashes.unshift(report);
    if (this.crashes.length > MAX_STORED_CRASHES) {
      this.crashes = this.crashes.slice(0, MAX_STORED_CRASHES);
    }
    this.persist();
    console.error(`[DIAGNOSTICS] Client ${type}:`, sanitizedMsg);
  }

  sanitize(raw: string): string {
    if (!raw) return '';
    return raw
      .replace(
        /(\/home\/[^/:\s"']+|\/Users\/[^/:\s"']+|[a-zA-Z]:\\Users\\[^\\/:\s"']+)/g,
        '[REDACTED_PATH]'
      )
      .replace(/\bacc_[0-9A-Za-z]{10,32}\b/g, '[ACCOUNT_ID_MASKED]')
      .replace(
        /\b(Assets|Liabilities|Income|Expenses|Equity|Aset|Liabilitas|Kewajiban|Ekuitas|Pendapatan|Pengeluaran|Beban)(?:(?::[-a-zA-Z0-9_]+)+|(?:\s*>\s*[A-Z0-9][-a-zA-Z0-9_&]*(?:\s+(?:&|on|of|[A-Z0-9][-a-zA-Z0-9_&]*))*)+)/g,
        '[ACCOUNT_MASKED]'
      )
      .replace(
        /(?:\b(USD|IDR|EUR|GBP|SGD|JPY|CAD|AUD|CHF|CNY|HKD|NZD|KRW|Rp)\b|[$€£¥])/gi,
        '[CURRENCY_MASKED]'
      )
      .replace(
        /-?\b\d{1,3}(?:[.,]\d{3})*(?:[.,]\d{2,})?\b|-?\b\d+([.,]\d{2,})?\b/g,
        '[NUMERIC_MASKED]'
      );
  }

  clear() {
    this.crashes = [];
    this.persist();
  }
}

export const diagnosticsState = new DiagnosticsStore();
