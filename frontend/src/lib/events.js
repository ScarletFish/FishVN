import { DEV_HTTP, DEV_HTTP_BASE } from './env.js';

let source = null;

export async function subscribe(name, handler) {
  if (DEV_HTTP) {
    if (!source) source = new EventSource(`${DEV_HTTP_BASE}/events`);
    const wrapped = (e) => handler(JSON.parse(e.data));
    source.addEventListener(name, wrapped);
    return () => source.removeEventListener(name, wrapped);
  }
  const { listen } = await import('@tauri-apps/api/event');
  const unlisten = await listen(name, (event) => handler(event.payload));
  return () => unlisten();
}
