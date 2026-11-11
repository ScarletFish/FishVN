// @vitest-environment jsdom
// 状态板只跑在 dev server 上，用冒烟测试保证它没有随组件契约漂移而挂掉。
import { afterEach, expect, test } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

import Board from './Board.svelte';

let app = null;

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

test('状态板能挂载：三列侧栏、两种日志汇总、toast 按钮都在', () => {
  document.body.innerHTML = '<div id="app"></div><div id="modal-root"></div>';
  app = mount(Board, { target: document.getElementById('app') });
  flushSync();

  expect(document.querySelectorAll('.board__bar')).toHaveLength(3);
  expect(document.querySelectorAll('.nav-btn__dot')).toHaveLength(2);
  expect(document.body.textContent).toContain('成功 58/62 部，2 部未匹配，2 部失败');
  expect(document.body.textContent).toContain('已处理 40/62 部');
  expect([...document.querySelectorAll('.board__chip')].map((el) => el.textContent.trim())).toEqual([
    '浅色',
    '深色',
    '绯红',
    '琥珀',
    '海蓝',
    '樱粉',
    '翠绿',
    '紫',
    '成功',
    '取消',
  ]);
});
