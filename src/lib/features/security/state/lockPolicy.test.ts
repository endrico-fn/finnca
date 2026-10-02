import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$lib/core/ipc/bindings', () => ({
  getBootId: vi.fn(),
}));

vi.mock('$lib/core/state/prefs', () => ({
  getPref: vi.fn(() => '15'),
  setPref: vi.fn(),
}));

import { LockPolicyStore } from './lockPolicy.svelte';
import { getBootId } from '$lib/core/ipc/bindings';

describe('LockPolicyStore', () => {
  let store: LockPolicyStore;

  beforeEach(() => {
    vi.clearAllMocks();
    store = new LockPolicyStore();
  });

  it('calculates inactivity timeout correctly', () => {
    store.timeoutMinutes = '15';
    expect(store.getInactivityTimeoutMs()).toBe(15 * 60 * 1000);

    store.timeoutMinutes = 'never';
    expect(store.getInactivityTimeoutMs()).toBe(Infinity);

    store.timeoutMinutes = '0';
    expect(store.getInactivityTimeoutMs()).toBe(15 * 60 * 1000);
  });

  it('bypasses fast-unlock check if mode is not on-reboot', async () => {
    store.mode = 'always';
    const result = await store.checkBootIdFastUnlock('any-boot-id');
    expect(result).toBe(true);
    expect(getBootId).not.toHaveBeenCalled();
  });

  it('requires unlock when boot ID does not match on-reboot, and does not bypass on subsequent calls', async () => {
    store.mode = 'on-reboot';
    vi.mocked(getBootId).mockResolvedValue('current-os-boot-id-1234');

    // Mismatched saved boot ID
    const firstCheck = await store.checkBootIdFastUnlock('different-saved-boot-id-9999');
    expect(firstCheck).toBe(false);

    // CRITICAL SECURITY ASSERTION: Second call MUST NOT return true!
    const secondCheck = await store.checkBootIdFastUnlock('different-saved-boot-id-9999');
    expect(secondCheck).toBe(false);
  });

  it('succeeds when boot ID matches, and resets on resetBootIdCheck', async () => {
    store.mode = 'on-reboot';
    vi.mocked(getBootId).mockResolvedValue('match-boot-id-42');

    const firstCheck = await store.checkBootIdFastUnlock('match-boot-id-42');
    expect(firstCheck).toBe(true);

    store.resetBootIdCheck();
    vi.mocked(getBootId).mockResolvedValue('new-reboot-id-77');

    const afterRebootCheck = await store.checkBootIdFastUnlock('match-boot-id-42');
    expect(afterRebootCheck).toBe(false);
  });
});
