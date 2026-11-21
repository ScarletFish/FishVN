// @vitest-environment jsdom
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

vi.mock('../lib/pick-path.js', () => ({
  pickExecutable: vi.fn(async () => 'E:\\Games\\Foo\\foo.exe'),
}));

vi.mock('../lib/api.js', () => ({
  api: {
    dirSize: vi.fn(async () => 1024 * 1024 * 1024),
    probeSteamAppid: vi.fn(async () => '123456'),
    listInstallations: vi.fn(async () => []),
    saveInstallations: vi.fn(async () => []),
  },
}));

import InstallationList from './InstallationList.svelte';
import { api } from '../lib/api.js';

let app = null;

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
  vi.clearAllMocks();
});

const settle = async () => {
  await new Promise((r) => setTimeout(r, 0));
  await new Promise((r) => setTimeout(r, 0));
  flushSync();
};

const draft = (extra = {}) => ({
  key: 'k1',
  path: 'E:\\Games\\Bar\\bar.exe',
  dirPath: 'E:\\Games\\Bar',
  entryPath: 'bar.exe',
  sizeBytes: null,
  leLaunch: false,
  magpie: 'off',
  launchArgs: '',
  steamLaunch: false,
  steamAppid: '',
  ...extra,
});

function render(props) {
  const modalRoot = document.createElement('div');
  modalRoot.id = 'modal-root';
  const target = document.createElement('div');
  document.body.append(modalRoot, target);
  app = mount(InstallationList, { target, props });
  flushSync();
  return target;
}

async function pickFromAddMenu(target, label) {
  const add = [...target.querySelectorAll('button')].find((b) =>
    b.textContent.includes('添加启动项'),
  );
  add.click();
  await settle();
  const item = [...document.querySelectorAll('.menu__item')].find((el) =>
    el.textContent.includes(label),
  );
  item.click();
  await settle();
}

test('打开表单不统计目录大小，缺大小的项不显示角标', async () => {
  const target = render({ items: [draft()], defaultKey: 'k1' });
  await settle();

  expect(api.dirSize).not.toHaveBeenCalled();
  expect(target.querySelector('.path-field__suffix')).toBeNull();
});

test('从菜单添加普通启动项后统计该目录并显示角标', async () => {
  const target = render({ items: [], defaultKey: null });

  await pickFromAddMenu(target, '添加启动项');

  expect(api.dirSize).toHaveBeenCalledWith('E:\\Games\\Foo');
  expect(api.probeSteamAppid).not.toHaveBeenCalled();
  expect(target.querySelector('.path-field__suffix')?.textContent.trim()).toBe('1.0 GB');
});

test('从菜单添加 Steam 启动项后反查 appid', async () => {
  const target = render({ items: [], defaultKey: null });

  await pickFromAddMenu(target, '添加 Steam 启动项');

  expect(api.probeSteamAppid).toHaveBeenCalledWith('E:\\Games\\Foo\\foo.exe');
  expect(target.querySelector('.path-field__suffix')?.textContent.trim()).toBe('1.0 GB');
});

test('Steam 启动项反查不到时路径下方报查不到且不给提交', async () => {
  api.probeSteamAppid.mockResolvedValueOnce(null);
  const target = render({ items: [], defaultKey: null });

  await pickFromAddMenu(target, '添加 Steam 启动项');

  expect(target.querySelector('.form-hint--error')?.textContent).toContain('没找到 Steam 游戏');
});

test('Steam 启动项不显示转区与启动参数，但保留 Magpie', async () => {
  const target = render({
    items: [draft({ steamLaunch: true, steamAppid: '123456' })],
    defaultKey: 'k1',
  });
  await settle();

  expect(target.querySelectorAll('.toggle-switch')).toHaveLength(1);
  expect(target.querySelector('.install-args')).toBeNull();
  expect(target.querySelector('.install-magpie')).not.toBeNull();
});

test('改了路径后按新目录重算', async () => {
  const target = render({ items: [draft({ sizeBytes: 5 * 1024 * 1024 * 1024 })], defaultKey: 'k1' });
  await settle();
  expect(api.dirSize).not.toHaveBeenCalled();

  target.querySelector('.path-field__browse').click();
  await settle();

  expect(api.dirSize).toHaveBeenCalledWith('E:\\Games\\Foo');
  expect(target.querySelector('.path-field__suffix')?.textContent.trim()).toBe('1.0 GB');
});
