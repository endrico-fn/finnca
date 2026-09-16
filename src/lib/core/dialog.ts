import { open, save } from '@tauri-apps/plugin-dialog';

export async function pickDirectory(): Promise<string | null> {
  const dir = await open({ directory: true, multiple: false });
  return typeof dir === 'string' ? dir : null;
}

export async function pickSaveFile(
  defaultPath?: string,
  filters?: Array<{ name: string; extensions: string[] }>
): Promise<string | null> {
  const filePath = await save({ defaultPath, filters });
  return typeof filePath === 'string' ? filePath : null;
}
