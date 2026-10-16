import { convertFileSrc } from '@tauri-apps/api/core';
import { api } from './api.js';

// 授权不能省：asset 协议白名单默认为空，不放行则 <img> 拿到 403 空响应
const cache = new Map();
const inFlight = new Map();

export function coverSrc(path) {
  if (!path) return Promise.resolve(null);

  const hit = cache.get(path);
  if (hit) return Promise.resolve(hit);

  let job = inFlight.get(path);
  if (!job) {
    job = api
      .thumbnail(path)
      .then((thumb) => api.allowLocalFile(thumb).then(() => convertFileSrc(thumb)))
      .then((url) => {
        cache.set(path, url);
        return url;
      })
      .finally(() => inFlight.delete(path));
    inFlight.set(path, job);
  }
  return job;
}