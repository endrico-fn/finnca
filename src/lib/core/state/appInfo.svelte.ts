import { APP_VERSION, getRuntimeVersion } from '$lib/core/types';

class AppInfoStore {
  version = $state(APP_VERSION);

  constructor() {
    if (typeof window !== 'undefined') {
      getRuntimeVersion().then((v) => {
        this.version = v;
      });
    }
  }
}

export const appInfo = new AppInfoStore();
