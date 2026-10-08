// 死 CSS 检查：类在 .svelte 的 class 属性、JS 字符串、或与其他类组成复合选择器里出现，都算有引用。
// 库注入的类名（如 overlayscrollbars 的 os-*）在 markup 里找不到字面量，只能靠复合选择器那条路兜住。
//
// 只警告不报错：先写样式后写 markup 是常见顺序，开发中途红会让人养成无视输出的习惯。
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const FRONTEND = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const SRC_DIR = join(FRONTEND, 'src');

const CLASS_RE = /\.(-?[A-Za-z_][\w-]*)/g;

function walk(dir, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name);
    if (entry.isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

/** 注释里的类名不是声明、@import 里的文件名也不是，照原样抽会把它们算成死 CSS */
function stripNonSelectors(css) {
  return css.replace(/\/\*[\s\S]*?\*\//g, '').replace(/@import[^;]+;/g, '');
}

const files = walk(SRC_DIR);
const cssFiles = files.filter((f) => extname(f) === '.css');
const codeFiles = files.filter((f) => extname(f) === '.svelte' || extname(f) === '.js');

const defined = new Map(); // 类 -> 声明所在文件
const referenced = new Set();

for (const file of cssFiles) {
  const css = stripNonSelectors(readFileSync(file, 'utf8'));

  for (const m of css.matchAll(CLASS_RE)) {
    if (!defined.has(m[1])) defined.set(m[1], file);
  }

  // 复合选择器里每个类都算彼此的引用，:has() 的参数也一样
  for (const m of css.matchAll(/([^{}]+)\{/g)) {
    const inSel = [...m[1].matchAll(CLASS_RE)].map((x) => x[1]);
    for (let i = 0; i < inSel.length; i++) {
      for (let j = 0; j < inSel.length; j++) {
        if (i !== j) referenced.add(inSel[i]);
      }
    }
  }
}

for (const file of codeFiles) {
  const code = readFileSync(file, 'utf8');
  for (const m of code.matchAll(/class[:=]\s*"([^"]*)"/g)) {
    for (const w of m[1].split(/\s+/)) if (w) referenced.add(w);
  }
  for (const m of code.matchAll(/'([^'\n]+)'/g)) referenced.add(m[1]);
  for (const m of code.matchAll(/`([^`\n]+)`/g)) {
    for (const w of m[1].split(/\s+/)) if (w) referenced.add(w);
  }
}

const dead = [...defined].filter(([cls]) => !referenced.has(cls));

if (dead.length) {
  console.log(`! 发现 ${dead.length} 处死 CSS（定义了但无引用）\n`);
  for (const [cls, file] of dead.sort()) {
    console.log(`  .${cls}  ${file.replace(FRONTEND + '\\', '')}`);
  }
  console.log('\n确认无用就删掉，提交前清理');
} else {
  console.log('✓ 无死 CSS');
}