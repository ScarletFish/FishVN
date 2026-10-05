// FishVN CSS token 检查 —— 强制 tokens.css 三层架构的纪律
//
//   1. var(--x) 必须已在 tokens.css 定义
//   2. 组件只能引用 L2 语义槽 / L3 尺度；引用 L1 原始值即报错
//      （组件直连原始色值 = 换主题时该组件不会跟着变，三层架构当场失效）
//   3. L2 块内禁止裸色值（hex / rgb() / hsl()），只能引用 L1 或用 color-mix() 合成
//   4. 组件 CSS 与 Svelte <style> 内禁止裸色值
//
// 已知缺口（待有组件后补）：间距/字号的写死值尚未检查，因为还没有可校准的组件样本。
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const FRONTEND = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const TOKENS_FILE = join(FRONTEND, 'src', 'css', 'tokens.css');
const SRC_DIR = join(FRONTEND, 'src');

const SECTION_MARKER = /===\s*L([123])\b/;
const DEF_RE = /(--[a-zA-Z0-9_-]+)\s*:/g;
const USE_RE = /var\(\s*(--[a-zA-Z0-9_-]+)/g;
const RAW_COLOR_RE = /#[0-9a-fA-F]{3,8}\b|\brgba?\s*\(|\bhsla?\s*\(/;

const errors = [];
const report = (file, line, msg) => errors.push(`${file}:${line}  ${msg}`);

function parseTokens() {
  const lines = readFileSync(TOKENS_FILE, 'utf8').split(/\r?\n/);
  const defs = new Map(); // name -> { layer, line }
  const l2Ranges = []; // L2 块的行号区间（1-based，闭区间）
  let layer = null;
  let l2Start = 0;

  lines.forEach((raw, i) => {
    const marker = raw.match(SECTION_MARKER);
    if (marker) {
      if (layer === 'L2') l2Ranges.push([l2Start, i]); // i 为 0-based，等于上一块的 1-based 末行
      layer = `L${marker[1]}`;
      if (layer === 'L2') l2Start = i + 2; // 跳过标记行
      return;
    }
    if (!layer) return;
    for (const m of raw.matchAll(DEF_RE)) {
      if (!defs.has(m[1])) defs.set(m[1], { layer, line: i + 1 });
    }
  });
  if (layer === 'L2') l2Ranges.push([l2Start, lines.length]);
  return { lines, defs, l2Ranges };
}

/** 逐行校验一段 CSS：var 引用必须已定义且不得指向 L1；禁止裸色值。 */
function checkSnippet(file, baseLine, code, defs) {
  code.split(/\r?\n/).forEach((raw, i) => {
    const lineNo = baseLine + i;
    if (RAW_COLOR_RE.test(raw)) {
      report(file, lineNo, '裸色值：请引用 L2 语义槽（L2 内可引用 L1 或用 color-mix 合成）');
    }
    for (const m of raw.matchAll(USE_RE)) {
      const def = defs.get(m[1]);
      if (!def) {
        report(file, lineNo, `未定义的 token ${m[1]}`);
      } else if (def.layer === 'L1') {
        report(file, lineNo, `${m[1]} 是 L1 原始值，组件只能连 L2/L3 语义槽`);
      }
    }
  });
}

function walk(dir, ext, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name);
    if (entry.isDirectory()) walk(p, ext, out);
    else if (p.endsWith(ext)) out.push(p);
  }
  return out;
}

const { lines, defs, l2Ranges } = parseTokens();

// 规则 3：L2 块内的裸色值
for (const [start, end] of l2Ranges) {
  for (let n = start; n <= end; n++) {
    const raw = lines[n - 1];
    if (raw && RAW_COLOR_RE.test(raw)) {
      report('src/css/tokens.css', n, 'L2 内禁止裸色值，只能引用 L1 或用 color-mix() 合成');
    }
  }
}

const files = new Set([...walk(SRC_DIR, '.css'), ...walk(SRC_DIR, '.svelte')]);
let scanned = 0;

for (const abs of files) {
  const file = relative(FRONTEND, abs).replaceAll('\\', '/');
  if (file === 'src/css/tokens.css') continue;
  const text = readFileSync(abs, 'utf8');

  if (file.endsWith('.svelte')) {
    for (const m of text.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)) {
      const startLine = text.slice(0, m.index).split(/\r?\n/).length;
      checkSnippet(file, startLine, m[1], defs);
      scanned++;
    }
  } else {
    checkSnippet(file, 1, text, defs);
    scanned++;
  }
}

const counts = { L1: 0, L2: 0, L3: 0 };
for (const { layer } of defs.values()) counts[layer]++;

if (errors.length) {
  console.error(`✗ CSS token 检查未通过（${errors.length} 项）\n`);
  for (const e of errors) console.error(`  ${e}`);
  process.exit(1);
}

console.log(
  `✓ CSS token 检查通过 · token ${defs.size} 个（L1 ${counts.L1} / L2 ${counts.L2} / L3 ${counts.L3}）· 扫描 ${scanned} 处样式`,
);