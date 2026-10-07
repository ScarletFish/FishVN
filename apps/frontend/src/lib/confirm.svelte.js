import { mount } from 'svelte';
import ConfirmDialog from '../components/ConfirmDialog.svelte';

// Svelte 5 的 mount 实例没有 props setter（那是 Svelte 4 的 legacy API），
// 改 props 要动当初传进 mount 的那个 $state 对象。
// 所以 state 与挂载逻辑同住一个 .svelte.js —— .js 里用不了 rune。
const props = $state({ open: false, action: 'deleteGame', name: '' });

let instance = null;
let settle = null;

function handleResult(result) {
  const done = settle;
  settle = null;
  done?.(result);
}

/**
 * 弹出确认框，resolve 用户的选择：确定 true，取消/遮罩/Esc false。
 *
 * action 必须在 i18n-zh.js 有 confirm.<action>.{title,message,confirm} 三条词条，
 * 缺了就直接把 key 显示给用户 —— t() 缺失回落 key，这里刻意不做兜底。
 * @param {string} action 如 'deleteGame'
 * @param {string} [name]  填进消息里的作品名
 */
export function confirm(action, name = '') {
  // 同时只允许一个确认框。复用上一个 promise 会让两次不同操作的调用方
  // 拿到同一个答案 —— 删 A 的确认框弹出时又请求删 B，A 就会删掉 B。
  if (settle) throw new Error('confirm: 已有确认框打开');

  const promise = new Promise((resolve) => {
    settle = resolve;
  });

  if (!instance) {
    instance = mount(ConfirmDialog, {
      target: document.createElement('div'),
      props: {
        get open() {
          return props.open;
        },
        set open(v) {
          props.open = v;
        },
        get action() {
          return props.action;
        },
        set action(v) {
          props.action = v;
        },
        get name() {
          return props.name;
        },
        set name(v) {
          props.name = v;
        },
        onresolve: handleResult,
      },
    });
  }

  props.action = action;
  props.name = name;
  props.open = true;
  return promise;
}