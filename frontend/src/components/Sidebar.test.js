// @vitest-environment jsdom
// 圆点是纯视觉提示，行为由 backfill-indicator 决定；这里锁「哪一项有圆点、什么态、提示什么」与导航按钮的鼠标聚焦契约。
// 反例自检：任一 .nav-btn 去掉 onmousedown，「阻止默认聚焦」会变红；连 onclick 一起删，「点击仍能导航」会变红。
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

import Sidebar from './Sidebar.svelte';

let app = null;

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

const ITEMS = [
  { id: 'home', label: '首页', icon: 'house' },
  { id: 'backfill', label: '补全', icon: 'download', dot: 'running', tip: '正在补全 3/62' },
  { id: 'stats', label: '统计', icon: 'chart-bar', dot: 'done', tip: '成功 5/5 部' },
];

function render(props = {}) {
  const target = document.createElement('div');
  document.body.append(target);
  app = mount(Sidebar, { target, props: { items: ITEMS, current: 'home', ...props } });
  flushSync();
  return [...document.querySelectorAll('.nav-btn')];
}

test('只有带 dot 的导航项出圆点，完成态单独加类', () => {
  const [home, backfill, stats] = render();

  expect(home.querySelector('.nav-btn__dot')).toBeNull();
  expect(backfill.querySelector('.nav-btn__dot').classList.contains('is-done')).toBe(false);
  expect(stats.querySelector('.nav-btn__dot').classList.contains('is-done')).toBe(true);
});

test('提示文案优先用状态，没有状态才回落到导航名', () => {
  const [home, backfill, stats] = render();

  expect(home.getAttribute('data-tip')).toBe('首页');
  expect(backfill.getAttribute('data-tip')).toBe('正在补全 3/62');
  expect(stats.getAttribute('data-tip')).toBe('成功 5/5 部');
});

test('圆点不进可访问名，导航名保持稳定', () => {
  const [, backfill] = render();
  const dot = backfill.querySelector('.nav-btn__dot');

  expect(dot.getAttribute('aria-hidden')).toBe('true');
  expect(backfill.getAttribute('aria-label')).toBe('补全');
});

test('收起态给侧栏加 collapsed 类并对辅助技术隐藏', () => {
  render({ collapsed: true });
  const sidebar = document.querySelector('.sidebar');

  expect(sidebar.classList.contains('collapsed')).toBe(true);
  expect(sidebar.getAttribute('aria-hidden')).toBe('true');
});

test('默认展开态没有 collapsed 类，也不隐藏', () => {
  render();
  const sidebar = document.querySelector('.sidebar');

  expect(sidebar.classList.contains('collapsed')).toBe(false);
  expect(sidebar.getAttribute('aria-hidden')).toBe('false');
});

test('鼠标按下侧栏按钮阻止默认聚焦，点击仍能导航', () => {
  const onnavigate = vi.fn();
  const [home] = render({
    onnavigate,
    bottom: [{ id: 'settings', label: '设置', icon: 'gear', onclick: vi.fn() }],
  });
  const buttons = [...document.querySelectorAll('.nav-btn')];

  expect(buttons).toHaveLength(4);
  for (const btn of buttons) {
    const ev = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
    btn.dispatchEvent(ev);
    expect(ev.defaultPrevented, '顶部与底部按钮都要阻止 mousedown 的默认聚焦').toBe(true);
  }

  home.click();
  expect(onnavigate).toHaveBeenCalledTimes(1);
});
