/** 角色的表单草稿与后端 payload 互转。
 *
 * 草稿里留着 `source`（原样取回的那一行），保存时铺在最前面、只覆盖表单里
 * 露出的那些字段 —— 本地编辑不露出来的 traits / 三围 / 生日等要靠它原样带回去，
 * 否则用户改一次名字就把 VNDB 同步来的考据数据抹了。
 */

import { toInteger } from './form-metadata.js';

export function newLocalId() {
  return `local-${crypto.randomUUID()}`;
}

function trimToNull(value) {
  const text = (value ?? '').trim();
  return text || null;
}

function numToText(value) {
  return value == null ? '' : String(value);
}

export function blankCharacter() {
  return {
    id: newLocalId(),
    source: null,
    name: '',
    original: '',
    gender: '',
    age: '',
    height: '',
    weight: '',
    role: '',
    imagePath: '',
    description: '',
    va: [],
  };
}

/** 性别在 VNDB 是按 release 逐条给的数组，这里只暴露一个值给用户选 */
export function toDraft(row = {}) {
  return {
    id: row.id ?? newLocalId(),
    source: row.id ? row : null,
    name: row.name ?? '',
    original: row.original ?? '',
    gender: row.gender?.[0] ?? '',
    age: numToText(row.age),
    height: numToText(row.height),
    weight: numToText(row.weight),
    role: row.role ?? '',
    imagePath: row.image_path ?? '',
    description: row.description ?? '',
    va: (row.va ?? []).map((entry) => ({
      key: newLocalId(),
      staffId: entry.staff_id ?? '',
      name: entry.name ?? '',
      note: entry.note ?? '',
    })),
  };
}

export function toPayload(draft) {
  return {
    ...(draft.source ?? {}),
    id: draft.id,
    name: trimToNull(draft.name),
    original: trimToNull(draft.original),
    gender: draft.gender ? [draft.gender] : null,
    age: toInteger(draft.age),
    height: toInteger(draft.height),
    weight: toInteger(draft.weight),
    role: trimToNull(draft.role),
    image_path: trimToNull(draft.imagePath),
    description: trimToNull(draft.description),
    va: draft.va.map((entry) => ({
      staff_id: entry.staffId || newLocalId(),
      name: trimToNull(entry.name),
      original: null,
      note: entry.note ?? '',
    })),
  };
}

export function blankVa() {
  return { key: newLocalId(), staffId: newLocalId(), name: '', note: '' };
}

/** 卡片名优先级与详情页一致：原名在前 */
export function displayName(draft) {
  return draft.original.trim() || draft.name.trim();
}

/** 无名角色存进去会占掉一个名字，撞上按角色名比对同名的 VNDB 同步（docs/dev/characters.md） */
function isUnnamed(draft) {
  return !displayName(draft);
}

export function namedOnly(list) {
  return list.filter((draft) => !isUnnamed(draft));
}

/** 顺序是角色的唯一真相，不另存序号 */
export function moveCharacters(list, index, delta) {
  const target = index + delta;
  if (target < 0 || target >= list.length) return list;
  const next = list.slice();
  [next[index], next[target]] = [next[target], next[index]];
  return next;
}
