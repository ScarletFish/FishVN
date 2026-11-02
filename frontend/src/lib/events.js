import { listen } from '@tauri-apps/api/event';
import { DEV_HTTP, DEV_HTTP_BASE } from './env.js';
import { ensureBackend } from './dev-backend.js';

let source = null;
let lastId = 0;
const handlers = new Map();

async function connect() {
  await ensureBackend();
  if (handlers.size === 0 || source) return;
  source = new EventSource(
    `${DEV_HTTP_BASE}/events${lastId > 0 ? `?since=${lastId + 1}` : ''}`,
  );
  for (const [name, map] of handlers) {
    for (const wrapped of map.values()) source.addEventListener(name, wrapped);
  }
  source.onerror = () => {
    source.close();
    source = null;
    if (handlers.size > 0) setTimeout(connect, 1000);
  };
}

function attach(name, handler) {
  if (!handlers.has(name)) handlers.set(name, new Map());
  const wrapped = (e) => {
    lastId = Number(e.lastEventId) || lastId;
    handler(JSON.parse(e.data));
  };
  handlers.get(name).set(handler, wrapped);
  if (source) source.addEventListener(name, wrapped);
}

function detach(name, handler) {
  const map = handlers.get(name);
  const wrapped = map?.get(handler);
  if (wrapped && source) source.removeEventListener(name, wrapped);
  map?.delete(handler);
  if (map && map.size === 0) handlers.delete(name);
}

export async function subscribe(name, handler) {
  if (DEV_HTTP) {
    await ensureBackend();
    attach(name, handler);
    if (!source) connect();
    return () => detach(name, handler);
  }
  const unlisten = await listen(name, (event) => handler(event.payload));
  return () => unlisten();
}
