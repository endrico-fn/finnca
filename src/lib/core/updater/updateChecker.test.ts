import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Update } from '@tauri-apps/plugin-updater';

let mockPrefs: Record<string, unknown> = {};

vi.mock('$lib/core/state/prefs', () => ({
  getPref: vi.fn((key: string, fallback: unknown) => {
    return mockPrefs[key] !== undefined ? mockPrefs[key] : fallback;
  }),
  setPref: vi.fn((key: string, value: unknown) => {
    mockPrefs[key] = value;
  }),
}));

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn(),
}));

import { check } from '@tauri-apps/plugin-updater';
import { checkAppUpdates, resetUpdateSessionState } from './updateChecker';
import { notificationState } from '$lib/core/state/notification.svelte';
import { updaterState } from '$lib/core/updater/updaterState.svelte';
import type { TranslationDict } from '$lib/core/i18n.svelte';

const mockCheck = vi.mocked(check);

const mockT = {
  updateAvailableTitle: 'UPDATE AVAILABLE',
  updateAvailableMsg: 'Finnca {version} is available.',
  updateActionView: 'VIEW RELEASE',
} as unknown as TranslationDict;

describe('checkAppUpdates', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockPrefs = {
      finnca_notif_toggles: { autoUpdate: true },
    };
    resetUpdateSessionState();
    notificationState.clearAll();
    updaterState.update = null;
    updaterState.phase = 'idle';
  });

  it('notifies user on startup when an update is available', async () => {
    const mockUpdate = {
      version: '1.0.1',
      currentVersion: '0.1.0',
      body: 'Release notes for 1.0.1',
      date: '2026-10-05T00:00:00Z',
      downloadAndInstall: vi.fn(),
    } as unknown as Update;

    mockCheck.mockResolvedValueOnce(mockUpdate);

    await checkAppUpdates(mockT);

    expect(mockCheck).toHaveBeenCalledTimes(1);
    expect(updaterState.update).toBe(mockUpdate);
    expect(updaterState.phase).toBe('available');
    expect(notificationState.notifications.length).toBe(1);

    const notif = notificationState.notifications[0];
    expect(notif.title).toBe('UPDATE AVAILABLE');
    expect(notif.message).toBe('Finnca v1.0.1 is available.');
    expect(notif.actionHref).toBe('app:update');
    expect(notif.actionLabel).toBe('VIEW RELEASE');
    expect(notif.priority).toBe('high');
  });

  it('does not duplicate notification in the same running session', async () => {
    const mockUpdate = {
      version: '1.0.1',
      currentVersion: '0.1.0',
      body: 'Notes',
      downloadAndInstall: vi.fn(),
    } as unknown as Update;

    mockCheck.mockResolvedValue(mockUpdate);

    await checkAppUpdates(mockT);
    expect(notificationState.notifications.length).toBe(1);

    // Call again in same session
    await checkAppUpdates(mockT);
    expect(notificationState.notifications.length).toBe(1);
  });

  it('re-notifies on subsequent app launch (new session) if update has not been performed', async () => {
    const mockUpdate = {
      version: '1.0.1',
      currentVersion: '0.1.0',
      body: 'Notes',
      downloadAndInstall: vi.fn(),
    } as unknown as Update;

    mockCheck.mockResolvedValue(mockUpdate);

    // First session (app open 1)
    await checkAppUpdates(mockT);
    expect(notificationState.notifications.length).toBe(1);

    // User closes app and reopens later (simulating app restart)
    resetUpdateSessionState();
    notificationState.clearAll();

    // Second session (app open 2)
    await checkAppUpdates(mockT);
    expect(notificationState.notifications.length).toBe(1);
    expect(notificationState.notifications[0].message).toContain('v1.0.1');
  });

  it('does not notify when no update is found', async () => {
    mockCheck.mockResolvedValueOnce(null);

    await checkAppUpdates(mockT);

    expect(notificationState.notifications.length).toBe(0);
    expect(updaterState.update).toBeNull();
  });

  it('respects autoUpdate toggle unless forced', async () => {
    mockPrefs.finnca_notif_toggles = { autoUpdate: false };
    const mockUpdate = { version: '1.0.1' } as unknown as Update;
    mockCheck.mockResolvedValue(mockUpdate);

    // Without force -> should not check
    await checkAppUpdates(mockT, false);
    expect(mockCheck).not.toHaveBeenCalled();

    // With force -> should check
    await checkAppUpdates(mockT, true);
    expect(mockCheck).toHaveBeenCalledTimes(1);
  });

  it('handles offline or network failures gracefully without throwing', async () => {
    mockCheck.mockRejectedValueOnce(new Error('Network error: failed to connect'));

    await expect(checkAppUpdates(mockT)).resolves.not.toThrow();
    expect(notificationState.notifications.length).toBe(0);
  });
});
