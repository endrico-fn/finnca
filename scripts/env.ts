declare global {
  var __APP_NAME__: string;
  var __APP_VERSION__: string;
  var $state: <T>(v: T) => T;
  var $derived: <T>(v: T) => T;
}

globalThis.__APP_NAME__ = 'finnca';
globalThis.__APP_VERSION__ = '0.1.0';
globalThis.$state = <T>(v: T): T => v;
globalThis.$derived = <T>(v: T): T => v;

export {};
