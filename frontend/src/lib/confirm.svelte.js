import { mount } from 'svelte';
import ConfirmDialog from '../components/ConfirmDialog.svelte';

// 用 .svelte.js 后缀：要 $state，.js 里用不了 rune
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
 * 缺了不做兜底：t() 会回落 key。
 * @param {string} action 如 'deleteGame'
 * @param {string} [name]  填进消息里的作品名
 */
export function confirm(action, name = '') {
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