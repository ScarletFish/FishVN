export const QUERY_DIMS = ['tags', 'status', 'rating', 'released', 'developer', 'length'];

export function emptyQueryUi() {
  return {
    tags: { open: false, enabled: true, mode: 'any', groups: [[]], exclude: [] },
    status: { open: false, enabled: true, values: [] },
    rating: { open: false, enabled: true, source: 'vndb', min: '', max: '' },
    released: { open: false, enabled: true, from: '', to: '' },
    developer: { open: false, enabled: true, name: '' },
    length: { open: false, enabled: true, source: 'vndb', min: '', max: '' },
  };
}

function num(value) {
  const n = Number(value);
  return value !== '' && value != null && Number.isFinite(n) ? n : null;
}

function ratingScale(value) {
  const n = num(value);
  return n === null ? null : n * 10;
}

function hoursToMinutes(value) {
  const n = num(value);
  return n === null ? null : n * 60;
}

export function dimFilled(dim, d) {
  switch (dim) {
    case 'tags':
      return d.groups.some((g) => g.length > 0) || d.exclude.length > 0;
    case 'status':
      return d.values.length > 0;
    case 'rating':
    case 'length':
      return num(d.min) !== null || num(d.max) !== null;
    case 'released':
      return Boolean(d.from || d.to);
    case 'developer':
      return Boolean(d.name.trim());
    default:
      return false;
  }
}

export function hasActive(ui) {
  return QUERY_DIMS.some((dim) => ui[dim].open);
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;

export function uiToSpec(ui) {
  const rows = [];
  if (ui.tags.enabled && dimFilled('tags', ui.tags)) {
    for (const group of ui.tags.groups) {
      if (group.length > 0) rows.push({ kind: 'tag_include', mode: ui.tags.mode, tags: [...group] });
    }
    if (ui.tags.exclude.length > 0) rows.push({ kind: 'tag_exclude', tags: [...ui.tags.exclude] });
  }
  if (ui.status.enabled && dimFilled('status', ui.status)) {
    rows.push({ kind: 'status', values: [...ui.status.values] });
  }
  if (ui.rating.enabled && dimFilled('rating', ui.rating)) {
    rows.push({
      kind: 'rating',
      source: ui.rating.source,
      min: ratingScale(ui.rating.min),
      max: ratingScale(ui.rating.max),
    });
  }
  if (ui.released.enabled && dimFilled('released', ui.released)) {
    rows.push({
      kind: 'released',
      from: ISO_DATE.test(ui.released.from) ? ui.released.from : null,
      to: ISO_DATE.test(ui.released.to) ? ui.released.to : null,
    });
  }
  if (ui.developer.enabled && dimFilled('developer', ui.developer)) {
    rows.push({ kind: 'developer', name: ui.developer.name.trim() });
  }
  if (ui.length.enabled && dimFilled('length', ui.length)) {
    rows.push({
      kind: 'length',
      source: ui.length.source,
      min: hoursToMinutes(ui.length.min),
      max: hoursToMinutes(ui.length.max),
    });
  }
  return { subject: 'game', rows };
}
