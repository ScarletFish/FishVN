// @vitest-environment jsdom
// 面板的搜索区只看有没有 vndb_id，不看跑批留下的 bind_source。
// 反例：把 showSearch 改回 bind_source === 'search'，两条会同时变红
// —— 未绑定那条拿不到搜索框，已绑定那条默认就展开了。
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

vi.mock('../../lib/api.js', () => ({
  api: {
    getGame: vi.fn(),
    getGameMetadata: vi.fn(),
    listGameCharacters: vi.fn(),
    searchVndb: vi.fn(),
    thumbnail: vi.fn(() => Promise.resolve('')),
    allowLocalFile: vi.fn(() => Promise.resolve()),
  },
}));

import BackfillPanel from './BackfillPanel.svelte';

let app = null;

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

function item(extra = {}) {
  return {
    game_id: 'g1',
    name: '千恋万花',
    vndb_id: null,
    state: null,
    bind_source: null,
    hit_title: null,
    hit_released: null,
    candidates: [],
    filled: [],
    characters: 0,
    error: null,
    ...extra,
  };
}

function render(props) {
  const target = document.createElement('div');
  document.body.append(target);
  app = mount(BackfillPanel, {
    target,
    props: {
      running: false,
      onchanged: () => {},
      onopenform: () => {},
      onopendetail: () => {},
      onclose: () => {},
      ...props,
    },
  });
  flushSync();
}

function searchBox() {
  return document.querySelector('.bf-search input');
}

function actionButton(label) {
  return [...document.querySelectorAll('button')].find(
    (btn) => btn.textContent.trim() === label,
  );
}

test('没有 vndb_id 的条目打开面板就能搜，搜索词预填游戏名', () => {
  render({ item: item() });

  expect(searchBox()?.value).toBe('千恋万花');
});

test('有 vndb_id 的条目默认不展开搜索，点换绑才展开', () => {
  render({ item: item({ vndb_id: 'v17821', state: 'ok', bind_source: 'search' }) });
  expect(searchBox()).toBeNull();

  actionButton('换绑').dispatchEvent(new MouseEvent('click', { bubbles: true }));
  flushSync();

  expect(searchBox()).not.toBeNull();
});
