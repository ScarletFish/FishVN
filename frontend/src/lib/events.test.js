import { afterEach, beforeEach, expect, test, vi } from 'vitest';

vi.mock('./env.js', () => ({ DEV_HTTP: true, DEV_HTTP_BASE: '/dev-http' }));

class FakeEventSource {
  static instances = [];

  constructor(url) {
    this.url = url;
    this.handlers = new Map();
    FakeEventSource.instances.push(this);
  }

  addEventListener(type, fn) {
    const set = this.handlers.get(type) ?? new Set();
    set.add(fn);
    this.handlers.set(type, set);
  }

  removeEventListener(type, fn) {
    this.handlers.get(type)?.delete(fn);
  }

  emit(type, payload) {
    const data = JSON.stringify(payload);
    for (const fn of this.handlers.get(type) ?? []) fn({ data });
  }
}

let subscribe;
let source;

beforeEach(async () => {
  FakeEventSource.instances = [];
  globalThis.EventSource = FakeEventSource;
  vi.resetModules();
  ({ subscribe } = await import('./events.js'));
  source = FakeEventSource.instances.at(-1);
});

afterEach(() => {
  delete globalThis.EventSource;
});

test('同一个连接服务多个订阅，不重复开', async () => {
  await subscribe('vndb-sync', () => {});
  await subscribe('library-scan', () => {});
  source = FakeEventSource.instances.at(-1);

  expect(FakeEventSource.instances).toHaveLength(1);
  expect(source.url).toBe('/dev-http/events');
});

test('订阅按事件名收各自的进度', async () => {
  const seen = [];
  await subscribe('vndb-sync', (e) => seen.push(e));
  source = FakeEventSource.instances.at(-1);

  source.emit('vndb-sync', { phase: 'images', done: 3, total: 20 });
  source.emit('library-scan', { phase: 'walk', done: 1, total: 9 });

  expect(seen).toEqual([{ phase: 'images', done: 3, total: 20 }]);
});

test('退订后不再收到', async () => {
  const seen = [];
  const off = await subscribe('vndb-sync', (e) => seen.push(e));
  source = FakeEventSource.instances.at(-1);

  source.emit('vndb-sync', { phase: 'meta' });
  off();
  source.emit('vndb-sync', { phase: 'characters' });

  expect(seen).toEqual([{ phase: 'meta' }]);
});
