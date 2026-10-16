/** 表单字符串 → 后端 payload 的字段映射。后端加字段或改类型时只动这里。 */

export function toNumber(raw) {
  const text = raw.trim();
  if (!text) return null;
  const value = Number(text);
  return Number.isFinite(value) ? value : null;
}

export function toInteger(raw) {
  const value = toNumber(raw);
  return value === null ? null : Math.trunc(value);
}

// titles 与 image_id 不在 fields 里也不能补：双层 Option 缺外层是「不动」，
// 写 null 反而会把同步来的本地化标题清掉。image_id 由封面下载管线自己写。
export function buildMetadataPatch(fields) {
  return {
    alias: fields.alias.trim() || null,
    released: fields.released.trim() || null,
    rating: toNumber(fields.rating),
    votecount: toInteger(fields.votecount),
    description: fields.description.trim() || null,
    image_path: fields.coverPath.trim() || null,
    tags: fields.tags,
    developers: fields.developers
      .split(',')
      .map((item) => item.trim())
      .filter(Boolean),
  };
}