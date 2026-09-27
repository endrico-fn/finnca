export type AppErrorCode =
  | 'ERR_DATABASE'
  | 'ERR_CRYPTO'
  | 'ERR_VAULT_LOCKED'
  | 'ERR_INVALID_INPUT'
  | 'ERR_NOT_FOUND'
  | 'ERR_CONFLICT'
  | 'ERR_PERIOD_LOCKED'
  | 'ERR_IO'
  | 'ERR_TIMEOUT'
  | 'ERR_UNKNOWN';

export interface StructuredAppError {
  code: AppErrorCode;
  message: string;
}

export class AppError extends Error {
  readonly code: AppErrorCode;

  constructor(code: AppErrorCode, message: string) {
    super(message);
    this.name = 'AppError';
    this.code = code;
  }

  override toString(): string {
    return this.message;
  }

  static fromUnknown(err: unknown, defaultMessage = 'An unexpected error occurred'): AppError {
    if (err instanceof AppError) {
      return err;
    }
    if (typeof err === 'object' && err !== null && 'code' in err && 'message' in err) {
      const e = err as StructuredAppError;
      return new AppError(e.code, e.message);
    }
    if (typeof err === 'string') {
      return new AppError('ERR_UNKNOWN', err);
    }
    if (err instanceof Error) {
      return new AppError('ERR_UNKNOWN', err.message);
    }
    return new AppError('ERR_UNKNOWN', defaultMessage);
  }
}

export function extractErrorMessage(err: unknown, defaultMessage = ''): string {
  if (err instanceof Error) {
    return err.message.replace(/^(\w*Error:\s*)+/i, '');
  }
  if (typeof err === 'string') {
    return err.replace(/^(\w*Error:\s*)+/i, '');
  }
  if (
    typeof err === 'object' &&
    err !== null &&
    'message' in err &&
    typeof (err as { message: unknown }).message === 'string'
  ) {
    return (err as { message: string }).message.replace(/^(\w*Error:\s*)+/i, '');
  }
  return String(err ?? defaultMessage).replace(/^(\w*Error:\s*)+/i, '');
}
