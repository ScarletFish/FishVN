import { mount } from 'svelte';
import ConfirmDialog from '../components/ConfirmDialog.svelte';

const props = $state({ open: false, action: 'deleteGame', params: {} });

let instance = null;
let settle = null;

function handleResult(result) {
  const done = settle;
  settle = null;
  done?.(result);
}

export function confirm(action, params = {}) {
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
        get params() {
          return props.params;
        },
        set params(v) {
          props.params = v;
        },
        onresolve: handleResult,
      },
    });
  }

  props.action = action;
  props.params = params;
  props.open = true;
  return promise;
}