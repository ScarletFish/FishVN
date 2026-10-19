import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url)).replace(/[\\/]+$/, '');
const DIRS = ['src-tauri', 'frontend', 'scripts'];
const SKIP = new Set(['node_modules', 'dist', 'target']);
const EXT = /\.(rs|js|mjs|ts|svelte|css)$/;

const WHOLE_FILE = new Set(['frontend/src/css/tokens.css']);
const VENDOR = /^frontend\/public\/vendor\//;
const TEST_FILE = [/^frontend\/src\/.*\.test\.js$/, /^src-tauri\/tests\//];
const DIRECTIVE = [
  /^<!--\s*svelte-ignore/,
  /^\/\/\s*@vitest-environment/,
  /^\/\/\s*@ts-(check|nocheck|ignore|expect-error)/,
  /^\/\/\s*(eslint|prettier)-(disable|ignore)/,
];

function walk(dir, files) {
  for (const name of readdirSync(dir)) {
    if (SKIP.has(name)) continue;
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, files);
    else if (EXT.test(name)) files.push(p);
  }
  return files;
}

const offenders = [];
let allowed = 0;

for (const rel of DIRS) {
  let files = [];
  try {
    files = walk(join(root, rel), []);
  } catch {
    continue;
  }
  for (const abs of files) {
    const path = abs.slice(root.length + 1).replace(/\\/g, '/');
    if (VENDOR.test(path)) continue;
    const isTest = TEST_FILE.some((re) => re.test(path));
    let blockEnd = null;
    readFileSync(abs, 'utf8')
      .split('\n')
      .forEach((line, i) => {
        const t = line.trim();
        let isComment = false;
        if (blockEnd) {
          isComment = true;
          if (t.includes(blockEnd)) blockEnd = null;
        } else if (t.startsWith('//')) {
          isComment = true;
        } else if (t.startsWith('/*')) {
          isComment = true;
          if (!t.includes('*/')) blockEnd = '*/';
        } else if (t.startsWith('<!--')) {
          isComment = true;
          if (!t.includes('-->')) blockEnd = '-->';
        }
        if (!isComment) return;
        const ok =
          WHOLE_FILE.has(path) ||
          isTest ||
          DIRECTIVE.some((re) => re.test(t));
        if (ok) {
          allowed += 1;
          return;
        }
        offenders.push(`${path}:${i + 1}  ${t}`);
      });
  }
}

if (offenders.length) {
  console.error(`✗ 发现 ${offenders.length} 行注释：`);
  for (const o of offenders) console.error('  ' + o);
  console.error('\n知识写进 docs/dev/* 或 docs/fdd-game-library.md（见 docs/dev/comments.md）。');
  console.error('确需留在代码里的例外，加进 scripts/check-comments.mjs 的白名单。');
  process.exit(1);
}

console.log(`✓ 无注释 · 白名单命中 ${allowed} 行`);
