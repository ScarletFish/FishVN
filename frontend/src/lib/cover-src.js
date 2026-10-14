import { convertFileSrc } from '@tauri-apps/api/core';
import { api } from './api.js';

// 本地图经 asset 协议渲染，而协议白名单默认是空的：不先授权，<img> 只会拿到 403 空响应
const granted = new Set();
const inFlight = new Map();

export function coverSrc(path) {
  if (!path) return Promise.resolve(null);
  if (granted.has(path)) return Promise.resolve(convertFileSrc(path));

  let job = inFlight.get(path);
  if (!job) {
    job = api
      .allowLocalFile(path)
      .then(() => {
        granted.add(path);
        return convertFileSrc(path);
      })
      .finally(() => inFlight.delete(path));
    inFlight.set(path, job);
  }
  return job;
}
