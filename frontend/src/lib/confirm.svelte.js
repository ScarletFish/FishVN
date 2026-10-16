import { mount } from 'svelte';
import ConfirmDialog from '../components/ConfirmDialog.svelte';

const props = $state({ open: false, action: 'deleteGame', name: '' });

let instance = null;
let settle = null;

function handleResult(result) {
  const done = settle;
  settle = null;
  done?.(result);
}

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