export function portal(node, target = '#modal-root') {
  const el = typeof target === 'string' ? document.querySelector(target) : target;
  if (el) el.appendChild(node);
  return {
    destroy() {
      node.remove();
    },
  };
}
