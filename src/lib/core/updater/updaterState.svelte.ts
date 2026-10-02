import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdaterPhase =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'verifying'
  | 'ready'
  | 'restarting'
  | 'up_to_date'
  | 'error';

interface DownloadProgress {
  downloaded: number;
  total: number | null;
}

class UpdaterState {
  phase = $state<UpdaterPhase>('idle');
  update = $state<Update | null>(null);
  progress = $state<DownloadProgress>({ downloaded: 0, total: null });
  errorMessage = $state<string | null>(null);
  screenOpen = $state(false);
  sessionStartedAt = $state<number | null>(null);

  get progressPercent(): number {
    if (!this.progress.total || this.progress.total === 0) return 0;
    return Math.min(100, Math.round((this.progress.downloaded / this.progress.total) * 100));
  }

  get downloadedMb(): string {
    return (this.progress.downloaded / 1_048_576).toFixed(1);
  }

  get totalMb(): string {
    if (!this.progress.total) return '?';
    return (this.progress.total / 1_048_576).toFixed(1);
  }

  get latestVersion(): string {
    return this.update?.version ?? '';
  }

  get currentVersion(): string {
    return this.update?.currentVersion ?? '';
  }

  get releaseNotes(): string {
    return this.update?.body ?? '';
  }

  get releaseDate(): string {
    return this.update?.date ?? '';
  }

  openScreen() {
    this.sessionStartedAt = Date.now();
    this.screenOpen = true;
  }

  closeScreen() {
    this.screenOpen = false;
    this.phase = 'idle';
    this.update = null;
    this.progress = { downloaded: 0, total: null };
    this.errorMessage = null;
    this.sessionStartedAt = null;
  }

  async checkForUpdate(): Promise<boolean> {
    this.phase = 'checking';
    this.errorMessage = null;
    try {
      const found = await check();
      if (!found) {
        this.phase = 'up_to_date';
        return false;
      }
      this.update = found;
      this.phase = 'available';
      return true;
    } catch (err) {
      this.phase = 'error';
      this.errorMessage = err instanceof Error ? err.message : String(err);
      return false;
    }
  }

  async downloadAndInstall(): Promise<void> {
    if (!this.update) return;
    this.phase = 'downloading';
    this.progress = { downloaded: 0, total: null };

    try {
      await this.update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            this.progress = { downloaded: 0, total: event.data.contentLength ?? null };
            break;
          case 'Progress':
            this.progress = {
              downloaded: this.progress.downloaded + event.data.chunkLength,
              total: this.progress.total,
            };
            break;
          case 'Finished':
            this.phase = 'verifying';
            break;
        }
      });

      this.phase = 'ready';
    } catch (err) {
      this.phase = 'error';
      this.errorMessage = err instanceof Error ? err.message : String(err);
    }
  }

  async applyAndRelaunch(): Promise<void> {
    this.phase = 'restarting';
    await relaunch();
  }
}

export const updaterState = new UpdaterState();
