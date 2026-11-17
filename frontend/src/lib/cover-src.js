import { convertFileSrc } from '@tauri-apps/api/core';
import { api } from './api.js';
import { DEV_HTTP, DEV_HTTP_BASE } from './env.js';

const cache = new Map();
const inFlight = new Map();

function httpFileUrl(path) {
  return `${DEV_HTTP_BASE}/file?path=${encodeURIComponent(path)}`;
}

function load(key, produce) {
  const hit = cache.get(key);
  if (hit) return Promise.resolve(hit);

  let job = inFlight.get(key);
  if (!job) {
    job = produce()
      .then((url) => {
        cache.set(key, url);
        return url;
      })
      .finally(() => inFlight.delete(key));
    inFlight.set(key, job);
  }
  return job;
}

export function coverSrc(path) {
  if (!path) return Promise.resolve(null);

  return load(path, () =>
    api.thumbnail(path).then((thumb) =>
      DEV_HTTP
        ? httpFileUrl(thumb)
        : api.allowLocalFile(thumb).then(() => convertFileSrc(thumb))
    )
  );
}

export function fullSrc(path) {
  if (!path) return Promise.resolve(null);

  return load(`full:${path}`, () =>
    DEV_HTTP
      ? Promise.resolve(httpFileUrl(path))
      : api.allowLocalFile(path).then(() => convertFileSrc(path))
  );
}
