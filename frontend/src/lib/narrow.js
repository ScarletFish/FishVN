const NARROW_MAIN_W = 1220;

export function useNarrowTier(el) {
  if (!el) return () => {};
  const root = document.documentElement;
  const apply = () => {
    if (el.clientWidth < NARROW_MAIN_W) root.dataset.narrow = 'on';
    else delete root.dataset.narrow;
  };
  const ro = new ResizeObserver(apply);
  ro.observe(el);
  apply();
  return () => ro.disconnect();
}
