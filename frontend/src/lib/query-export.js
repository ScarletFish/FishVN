import { QUERY_DIMS, CHAR_DIMS, dimFilled } from './query-spec.js';
import { tagLabel, traitLabel, tagLabelByName, traitKeyLabel } from './tag-label.svelte.js';
import { statusLabel } from './status.js';
import { DICT_ZH } from './i18n-zh.js';

const SUBJECT_LABEL = { game: '作品', character: '角色' };
const GENDER_LABELS = DICT_ZH.detail.gender;
const ROLE_LABELS = DICT_ZH.gameForm.charRole;
const TRAIT_GROUP_LABELS = DICT_ZH.detail.traitGroups;
const SEXUAL_TRAIT_GROUPS = new Set(['Engages in (Sexual)', 'Subject of (Sexual)']);

const defaultLabels = {
  tag: (v) => (typeof v === 'string' ? tagLabelByName(v) : tagLabel(v)),
  trait: (t) => (typeof t === 'string' ? traitKeyLabel(t) : traitLabel(t)),
  status: (k) => statusLabel(k) || k,
  gender: (k) => GENDER_LABELS[k] || k,
  role: (k) => ROLE_LABELS[k] || k,
  traitGroup: (k) => TRAIT_GROUP_LABELS[k] || k,
};

function escHeader(s) {
  return String(s).replace(/^#+/, (m) => '\\' + m);
}

function escValue(s) {
  return String(s).replace(/\r?\n/g, ' ').replace(/\|/g, '\\|');
}

function joinVas(s) {
  if (!s) return '';
  return String(s)
    .split(',')
    .map((v) => v.trim())
    .filter(Boolean)
    .join('、');
}

function fmtTime(d = new Date()) {
  const p = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

function tagPart(ui, key, label) {
  const parts = [];
  const mode = ui[key].mode === 'all' ? '含全部' : '含任一';
  for (const group of ui[key].groups || []) {
    const names = group.map((v) => label(v)).filter(Boolean);
    if (names.length) parts.push(`${mode}：${names.join('、')}`);
  }
  const excluded = (ui[key].exclude || []).map((v) => label(v)).filter(Boolean);
  if (excluded.length) parts.push(`排除：${excluded.join('、')}`);
  return parts.join('；');
}

function summarize(ui, labels) {
  const out = [];
  for (const dim of [...QUERY_DIMS, ...CHAR_DIMS]) {
    const d = ui[dim];
    if (!d || d.enabled === false || !dimFilled(dim, d)) continue;
    switch (dim) {
      case 'tags': {
        const t = tagPart(ui, 'tags', labels.tag);
        if (t) out.push(['标签', t]);
        break;
      }
      case 'traits': {
        const t = tagPart(ui, 'traits', labels.trait);
        if (t) out.push(['特性', t]);
        break;
      }
      case 'status':
        out.push(['状态', d.values.map((v) => labels.status(v)).join('、')]);
        break;
      case 'rating': {
        const src = d.source === 'vndb' ? 'VNDB 评分' : '个人评分';
        const b = [];
        if (d.min !== '') b.push(`≥ ${d.min}`);
        if (d.max !== '') b.push(`≤ ${d.max}`);
        if (b.length) out.push([src, b.join(' 且 ')]);
        break;
      }
      case 'length': {
        const src = d.source === 'vndb' ? '时长(VNDB)' : '时长(个人)';
        const b = [];
        if (d.min !== '') b.push(`≥ ${d.min}h`);
        if (d.max !== '') b.push(`≤ ${d.max}h`);
        if (b.length) out.push([src, b.join(' 且 ')]);
        break;
      }
      case 'released': {
        const b = [];
        if (d.from) b.push(`≥ ${d.from}`);
        if (d.to) b.push(`≤ ${d.to}`);
        if (b.length) out.push(['发行日期', b.join(' 且 ')]);
        break;
      }
      case 'developer':
        if (d.name.trim()) out.push(['开发商', d.name.trim()]);
        break;
      case 'va': {
        const names = typeof labels.va === 'function'
          ? d.values.map((v) => labels.va(v)).filter(Boolean)
          : [];
        out.push(['声优', names.length ? names.join('、') : `${d.values.length} 位`]);
        break;
      }
      case 'role':
        out.push(['戏份', d.values.map((v) => labels.role(v)).join('、')]);
        break;
      case 'gender':
        out.push(['性别', d.values.map((v) => labels.gender(v)).join('、')]);
        break;
      case 'charname':
        if (d.value.trim()) out.push(['角色名含', d.value.trim()]);
        break;
    }
  }
  return out;
}

function summarizeText(ui, labels) {
  const items = summarize(ui, labels);
  if (!items.length) return '未设条件';
  return items.map(([k, v]) => `${k}：${v}`).join('；');
}

function renderGame(g, L) {
  const lines = [];
  if (g.alias) lines.push(`别名：${escValue(g.alias)}`);
  if (g.vndb_id) lines.push(`VNDB：https://vndb.org/${escValue(g.vndb_id)}`);
  if (g.status) lines.push(`状态：${escValue(L.status(g.status))}`);
  if (typeof g.user_rating === 'number') lines.push(`我的评分：${(g.user_rating / 10).toFixed(1)}`);
  if (typeof g.vndb_rating === 'number') lines.push(`VNDB 评分：${(g.vndb_rating / 10).toFixed(2)}`);
  const va = joinVas(g.va_original_names);
  if (va) lines.push(`主要声优：${va}`);
  const title = escHeader(g.name || '未命名');
  return `## ${title}\n\n${lines.join('\n')}`;
}

function renderChar(c, L) {
  const title =
    escHeader(c.original || c.name || '未命名') +
    (c.game_title ? `（出自《${escValue(c.game_title)}》）` : '');
  const lines = [];
  if (c.role) lines.push(`戏份：${escValue(L.role(c.role))}`);
  if (Array.isArray(c.gender) && c.gender.length) {
    lines.push(`性别：${c.gender.map((x) => L.gender(x)).filter(Boolean).join('、')}`);
  }
  const traits = Array.isArray(c.traits) ? c.traits : [];
  const visibleTraits = traits.filter((t) => !SEXUAL_TRAIT_GROUPS.has(t.group_name));
  if (visibleTraits.length) {
    const groups = {};
    for (const t of visibleTraits) {
      const group = L.traitGroup(t.group_name || 'uncategorized');
      (groups[group] ||= []).push(L.trait(t));
    }
    const traitLines = Object.keys(groups).map(
      (group) => `- ${group}：${groups[group].filter(Boolean).join('、')}`,
    );
    lines.push(`特性：\n${traitLines.join('\n')}`);
  }
  return `## ${title}\n\n${lines.join('\n')}`;
}

export function buildQueryMarkdown({ subject = 'game', ui = {}, rows = [], labels, now } = {}) {
  const L = { ...defaultLabels, ...(labels || {}) };
  const subj = SUBJECT_LABEL[subject] || subject;
  const count = rows.length;
  const heading = `# FishVN 筛选结果 · ${subj}\n`;
  const intro = `以下是在筛选条件（${summarizeText(ui, L)}）下命中的 ${count} 个${subj}。导出时间：${fmtTime(now)}。\n`;
  const blocks = rows.map((row) =>
    subject === 'character' ? renderChar(row, L) : renderGame(row, L),
  );
  const sep = '\n\n---\n\n';
  return heading + intro + (blocks.length ? sep + blocks.join(sep) + '\n' : '\n');
}
