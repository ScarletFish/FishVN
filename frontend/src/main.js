import { mount } from 'svelte';
import './css/styles.css';
import App from './App.svelte';
import { useScrollbars } from './lib/scrollbars.js';
import { report } from './lib/frontend-log.js';

const isTauri = '__TAURI_INTERNALS__' in window;

window.addEventListener('error', (e) => {
  report('error', `未捕获异常：${e.message}（${e.filename}:${e.lineno}）`);
});
window.addEventListener('unhandledrejection', (e) => {
  report('error', `未处理的 Promise 拒绝：${e.reason}`);
});

const splash = window.__splash;

if (isTauri) {
  const readyTimeout = setTimeout(() => {
    report('warn', '8 秒兜底：app:ready 未按时到达');
    splash?.ready();
  }, 8000);
  window.addEventListener(
    'app:ready',
    () => {
      clearTimeout(readyTimeout);
      splash?.ready();
      const bootLabel = window.__reloaded === true ? '重载' : '启动';
      report('info', `${bootLabel}：列表就绪 ${Math.round(performance.now())}ms`);
    },
    { once: true }
  );
} else {
  document.getElementById('splash')?.remove();
}

const app = mount(App, {
  target: document.getElementById('app'),
});

useScrollbars(document.querySelector('.shell__main'));

export default app;
