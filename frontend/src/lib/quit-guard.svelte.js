const sources = new Map();

export const quitGuard = $state({ dirty: false });

export function markQuitDirty(source, dirty) {
  if (dirty) sources.set(source, true);
  else sources.delete(source);
  quitGuard.dirty = sources.size > 0;
}
