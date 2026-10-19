import { mount } from 'svelte';
import './css/styles.css';
import App from './App.svelte';
import { useScrollbars } from './lib/scrollbars.js';
import { report } from './lib/frontend-log.js';

window.addEventListener('error', (e) => {
  report('error', `未捕获异常：${e.message}（${e.filename}:${e.lineno}）`);
});
window.addEventListener('unhandledrejection', (e) => {
  report('error', `未处理的 Promise 拒绝：${e.reason}`);
});

const splash = document.getElementById('splash');

function hideSplash() {
  if (!splash || splash.classList.contains('is-hidden')) return;
  splash.classList.add('is-hidden');
  setTimeout(() => splash.remove(), 320);
}

if ('__TAURI_INTERNALS__' in window) {
  window.addEventListener(
    'app:ready',
    () => {
      hideSplash();
      report('info', `启动：webview 导航到列表就绪 ${Math.round(performance.now())}ms`);
    },
    { once: true }
  );
  setTimeout(() => {
    if (splash && !splash.classList.contains('is-hidden')) {
      report('warn', '8 秒兜底隐藏 splash：app:ready 未按时到达');
    }
    hideSplash();
  }, 8000);
} else {
  splash?.remove();
}

const app = mount(App, {
  target: document.getElementById('app'),
});

useScrollbars(document.querySelector('.shell__main'));

export default app;
