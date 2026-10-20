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
  window.addEventListener(
    'app:ready',
    () => {
      splash?.ready();
      report('info', `启动：列表就绪 ${Math.round(performance.now())}ms`);
    },
    { once: true }
  );
  setTimeout(() => {
    report('warn', '8 秒兜底：app:ready 未按时到达');
    splash?.ready();
  }, 8000);
} else {
  document.getElementById('splash')?.remove();
}

const app = mount(App, {
  target: document.getElementById('app'),
});

useScrollbars(document.querySelector('.shell__main'));

export default app;
