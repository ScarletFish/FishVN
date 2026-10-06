import { readdirSync, statSync } from 'node:fs';
import type { Dirent } from 'node:fs';
import { extname, join, resolve } from 'node:path';

/** 双击即可运行的扩展名 */
const EXECUTABLE_EXT = new Set(['.exe', '.bat', '.cmd']);

/** 顶层记第 0 层，候选入口最深收到第 2 层，即两层子目录 */
const MAX_EXE_DEPTH = 3;

/** 候选入口最多留几个，超出部分丢弃 */
const MAX_EXES = 30;

export interface DirInfo {
  sizeBytes: number;
  /** 候选入口，相对 dirPath，层数浅的在前 */
  exes: string[];
}

/**
 * 一次遍历同时算出目录大小与候选入口。
 * 大小累加全部深度；候选入口只收 MAX_EXE_DEPTH 层以内。
 * 符号链接的 isDirectory() 为 false，天然不进递归。
 *
 * 大小是同步全深度遍历，单个目录（几个GB）可接受。批量遍历要另走异步并带进度。
 */
export function readDirInfo(dirPath: string): DirInfo {
  let sizeBytes = 0;
  const found: { depth: number; rel: string }[] = [];

  const walk = (current: string, rel: string, depth: number): void => {
    let entries: Dirent[];
    try {
      entries = readdirSync(current, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const childRel = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        walk(join(current, entry.name), childRel, depth + 1);
        continue;
      }
      if (!entry.isFile()) continue;
      try {
        sizeBytes += statSync(join(current, entry.name)).size;
      } catch {
        continue;
      }
      if (depth < MAX_EXE_DEPTH && EXECUTABLE_EXT.has(extname(entry.name).toLowerCase())) {
        if (found.length < MAX_EXES) found.push({ depth, rel: childRel });
      }
    }
  };

  walk(dirPath, '', 0);
  found.sort((a, b) => a.depth - b.depth || a.rel.localeCompare(b.rel));

  return { sizeBytes, exes: found.map((f) => f.rel) };
}

export type DirInspection = { dirPath: string; info: DirInfo } | { error: string };

/** 目录必须存在且是目录；路径原样解析成绝对写法 */
export function inspectDir(input: string): DirInspection {
  const dirPath = resolve(input);
  let stat;
  try {
    stat = statSync(dirPath);
  } catch {
    return { error: '目录不存在' };
  }
  if (!stat.isDirectory()) return { error: '所选路径不是目录' };
  return { dirPath, info: readDirInfo(dirPath) };
}