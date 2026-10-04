import { resolve } from 'node:path';

// 编译产物固定落在 <ROOT>/dist/server/lib/，据此回溯到项目根
export const ROOT = resolve(import.meta.dirname, '../../..');

/** 前端构建产物（vite outDir 指向此处） */
export const WEB_DIR = resolve(ROOT, 'dist/web');

/** 运行时数据：数据库等 */
export const DATA_DIR = resolve(ROOT, 'data');