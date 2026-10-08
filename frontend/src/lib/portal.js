// 把弹窗节点搬到 body 级的 #modal-root，与 #app 平级 —— 背景层靠 inert 屏蔽交互，
// 两者放成父子关系会把弹窗自己一起屏蔽掉。
export function portal(node, target = '#modal-root') {
  const el = typeof target === 'string' ? document.querySelector(target) : target;
  if (el) el.appendChild(node);
  return {
    destroy() {
      node.remove();
    },
  };
}
