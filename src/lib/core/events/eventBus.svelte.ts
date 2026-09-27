export type AppEventPayloads = {
  'vault:unlocked': { vaultName: string; username: string };
  'vault:locked': void;
  'vault:registry_changed': void;
  'vault:deleted': void;
  'transaction:posted': { id: string };
  'accounts:changed': void;
  'activity:touched': void;
  'modal:open': { modalId: string };
  'fx:rate_changed': { rate: number };
};

export type AppEventName = keyof AppEventPayloads;

type EventHandler<T> = (payload: T) => void;

type AnyHandler = (payload: never) => void;

class EventBus {
  private handlers = new Map<string, Set<AnyHandler>>();

  on<K extends AppEventName>(event: K, handler: EventHandler<AppEventPayloads[K]>): () => void {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, new Set());
    }
    const set = this.handlers.get(event)!;
    const raw = handler as unknown as AnyHandler;
    set.add(raw);
    return () => {
      set.delete(raw);
      if (set.size === 0) {
        this.handlers.delete(event);
      }
    };
  }

  emit<K extends AppEventName>(event: K, payload: AppEventPayloads[K]): void {
    const set = this.handlers.get(event);
    if (!set) return;
    for (const handler of set) {
      try {
        (handler as unknown as EventHandler<AppEventPayloads[K]>)(payload);
      } catch (err) {
        console.error(`[EventBus] Error in handler for event "${String(event)}":`, err);
      }
    }
  }
}

export const eventBus = new EventBus();
