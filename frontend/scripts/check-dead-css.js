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

  for (const m of css.matchAll(/([^{}]+)\{/g)) {
    const inSel = [...m[1].matchAll(CLASS_RE)].map((x) => x[1]);
    for (const cls of inSel) {
      if (!defined.has(cls)) defined.set(cls, file);
    }
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
  for (const m of code.matchAll(/class:([\w-]+)/g)) referenced.add(m[1]);
  for (const m of code.matchAll(/class=\{([^}]*)\}/g)) {
    for (const q of m[1].matchAll(/['"]([^'"]+)['"]/g)) {
      for (const w of q[1].split(/\s+/)) if (w) referenced.add(w);
    }
  }
  for (const m of code.matchAll(/\bvariant="([^"]+)"/g)) referenced.add(m[1]);
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