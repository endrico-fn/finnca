import { page } from '$app/state';
import { SvelteURL } from 'svelte/reactivity';

export class TabRouter<T extends string> {
  #current = $state<T>() as T;
  #paramKey: string;
  #validTabs: readonly T[];

  constructor(defaultTab: T, validTabs: readonly T[], paramKey: string = 'tab') {
    this.#validTabs = validTabs;
    this.#paramKey = paramKey;

    const initial = page.url?.searchParams.get(paramKey) as T | null;
    this.#current = initial && validTabs.includes(initial) ? initial : defaultTab;
  }

  get current(): T {
    return this.#current;
  }

  set current(tab: T) {
    this.setTab(tab);
  }

  setTab(tab: T) {
    if (!this.#validTabs.includes(tab)) return;
    this.#current = tab;
    if (typeof window !== 'undefined') {
      const url = new SvelteURL(window.location.href);
      url.searchParams.set(this.#paramKey, tab);
      window.history.replaceState({}, '', url.toString());
    }
  }
}

export function createTabRouter<T extends string>(
  defaultTab: T,
  validTabs: readonly T[],
  paramKey: string = 'tab'
): TabRouter<T> {
  return new TabRouter(defaultTab, validTabs, paramKey);
}
