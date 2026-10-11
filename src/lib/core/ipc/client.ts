import { invoke } from '@tauri-apps/api/core';
import { AppError } from './errors';

export interface InvokeOptions {
  timeoutMs?: number;
  label?: string;
}

export async function invokeIpc<T>(
  command: string,
  args?: Record<string, unknown>,
  options: InvokeOptions = {}
): Promise<T> {
  const timeoutMs = options.timeoutMs ?? 30_000;
  const label = options.label ?? command;

  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeoutPromise = new Promise<never>((_, reject) => {
    timer = setTimeout(() => {
      reject(new AppError('ERR_TIMEOUT', `Command '${label}' timed out after ${timeoutMs}ms`));
    }, timeoutMs);
  });

  try {
    const invokePromise = invoke<T>(command, args);
    const result = await Promise.race([invokePromise, timeoutPromise]);
    return result;
  } catch (err) {
    throw AppError.fromUnknown(err, `Command '${label}' failed`);
  } finally {
    if (timer !== undefined) {
      clearTimeout(timer);
    }
  }
}
