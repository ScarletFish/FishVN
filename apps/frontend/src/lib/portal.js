// 把弹窗节点搬到 body 级的 #modal-root：脱离视图容器的 overflow 与 stacking context；
// 背景层 #app 靠 inert 屏蔽交互，两者必须是 body 下的兄弟节点。
export function portal(node, target = '#modal-root') {
  const el = typeof target === 'string' ? document.querySelector(target) : target;
  if (el) el.appendChild(node);
  return {
    destroy() {
      node.remove();
    },
  };
}
