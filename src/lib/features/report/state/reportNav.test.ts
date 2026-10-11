import { describe, it, expect } from 'vitest';
import {
  REPORT_GROUPS,
  REPORT_TAB_LABELS,
  REPORT_SHORTCUTS,
  ORDERED_REPORT_TABS,
  reportGroupOf,
} from './reportNav';
import { en } from '$lib/core/i18n/en';
import { id } from '$lib/core/i18n/id';
import type { ReportTab } from './report.svelte';

describe('reportNav configuration & 3-group navigation architecture', () => {
  it('defines exactly 3 top-level category groups', () => {
    expect(REPORT_GROUPS).toHaveLength(3);
    expect(REPORT_GROUPS.map((g) => g.id)).toEqual([
      'statements',
      'liquidity',
      'forecast-intelligence',
    ]);
  });

  it('assigns the correct tabs to Statements category', () => {
    const statementsGroup = REPORT_GROUPS.find((g) => g.id === 'statements');
    expect(statementsGroup).toBeDefined();
    expect(statementsGroup?.defaultTab).toBe('tb');
    expect(statementsGroup?.tabs).toEqual(['tb', 'pnl', 'bs']);
  });

  it('assigns the correct tabs to Cash & Liquidity category', () => {
    const liquidityGroup = REPORT_GROUPS.find((g) => g.id === 'liquidity');
    expect(liquidityGroup).toBeDefined();
    expect(liquidityGroup?.defaultTab).toBe('cashflow');
    expect(liquidityGroup?.tabs).toEqual(['cashflow', 'income-exp', 'spending']);
  });

  it('assigns the correct tabs to Forecast & Intelligence category', () => {
    const forecastGroup = REPORT_GROUPS.find((g) => g.id === 'forecast-intelligence');
    expect(forecastGroup).toBeDefined();
    expect(forecastGroup?.defaultTab).toBe('forecast');
    expect(forecastGroup?.tabs).toEqual(['forecast', 'debt', 'trends', 'fx', 'networth']);
  });

  it('covers all 11 valid tabs across the 3 categories without duplicates or omissions', () => {
    const allGroupTabs = REPORT_GROUPS.flatMap((g) => g.tabs);
    expect(allGroupTabs).toHaveLength(11);
    expect(new Set(allGroupTabs).size).toBe(11);
    expect(allGroupTabs.sort()).toEqual([...ORDERED_REPORT_TABS].sort());
  });

  it('correctly maps each tab to its category via reportGroupOf', () => {
    expect(reportGroupOf('tb')).toBe('statements');
    expect(reportGroupOf('pnl')).toBe('statements');
    expect(reportGroupOf('bs')).toBe('statements');

    expect(reportGroupOf('cashflow')).toBe('liquidity');
    expect(reportGroupOf('income-exp')).toBe('liquidity');
    expect(reportGroupOf('spending')).toBe('liquidity');

    expect(reportGroupOf('forecast')).toBe('forecast-intelligence');
    expect(reportGroupOf('debt')).toBe('forecast-intelligence');
    expect(reportGroupOf('trends')).toBe('forecast-intelligence');
    expect(reportGroupOf('fx')).toBe('forecast-intelligence');
    expect(reportGroupOf('networth')).toBe('forecast-intelligence');
  });

  it('provides valid unique shortcuts for all 11 tabs', () => {
    const shortcuts = Object.values(REPORT_SHORTCUTS);
    expect(shortcuts).toHaveLength(11);
    expect(new Set(shortcuts).size).toBe(11);

    expect(REPORT_SHORTCUTS.tb).toBe('1');
    expect(REPORT_SHORTCUTS.pnl).toBe('2');
    expect(REPORT_SHORTCUTS.bs).toBe('3');
    expect(REPORT_SHORTCUTS.cashflow).toBe('4');
    expect(REPORT_SHORTCUTS['income-exp']).toBe('5');
    expect(REPORT_SHORTCUTS.spending).toBe('6');
    expect(REPORT_SHORTCUTS.forecast).toBe('7');
    expect(REPORT_SHORTCUTS.debt).toBe('8');
    expect(REPORT_SHORTCUTS.trends).toBe('9');
    expect(REPORT_SHORTCUTS.fx).toBe('0');
    expect(REPORT_SHORTCUTS.networth).toBe('-');
  });

  it('guarantees every group labelKey exists in both en and id dictionaries', () => {
    for (const group of REPORT_GROUPS) {
      expect(en[group.labelKey]).toBeDefined();
      expect(en[group.labelKey].length).toBeGreaterThan(0);
      expect(id[group.labelKey]).toBeDefined();
      expect(id[group.labelKey].length).toBeGreaterThan(0);
    }
  });

  it('guarantees every tab labelKey exists in both en and id dictionaries', () => {
    for (const tab of ORDERED_REPORT_TABS) {
      const labelKey = REPORT_TAB_LABELS[tab as ReportTab];
      expect(labelKey).toBeDefined();
      expect(en[labelKey]).toBeDefined();
      expect(en[labelKey].length).toBeGreaterThan(0);
      expect(id[labelKey]).toBeDefined();
      expect(id[labelKey].length).toBeGreaterThan(0);
    }
  });

  it('aligns ORDERED_REPORT_TABS indices directly to keyboard shortcuts (1-9, 0, -)', () => {
    for (let i = 0; i < 9; i++) {
      const tab = ORDERED_REPORT_TABS[i];
      expect(REPORT_SHORTCUTS[tab]).toBe(String(i + 1));
    }
    expect(REPORT_SHORTCUTS[ORDERED_REPORT_TABS[9]]).toBe('0');
    expect(REPORT_SHORTCUTS[ORDERED_REPORT_TABS[10]]).toBe('-');
  });

  it('safely falls back to statements group for unrecognized tabs', () => {
    expect(reportGroupOf('unknown-tab' as ReportTab)).toBe('statements');
  });

  it('verifies superseded railNav keys are purged from dictionaries', () => {
    expect('railNavTitle' in en).toBe(false);
    expect('railNavShortcutHint' in en).toBe(false);
    expect('railNavTitle' in id).toBe(false);
    expect('railNavShortcutHint' in id).toBe(false);
  });
});
