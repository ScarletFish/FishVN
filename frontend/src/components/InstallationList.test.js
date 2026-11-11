// @vitest-environment jsdom
import { afterEach, expect, test, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

vi.mock('../lib/pick-path.js', () => ({
  pickExecutable: vi.fn(async () => 'E:\\Games\\Foo\\foo.exe'),
}));

vi.mock('../lib/api.js', () => ({
  api: {
    dirSize: vi.fn(async () => 1024 * 1024 * 1024),
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
  ...extra,
});

function render(props) {
  const target = document.createElement('div');
  document.body.append(target);
  app = mount(InstallationList, { target, props });
  flushSync();
  return target;
}

test('打开表单不统计目录大小，缺大小的项不显示角标', async () => {
  const target = render({ items: [draft()], defaultKey: 'k1' });
  await settle();

  expect(api.dirSize).not.toHaveBeenCalled();
  expect(target.querySelector('.path-field__suffix')).toBeNull();
});

test('添加启动项后统计该目录并显示角标', async () => {
  const target = render({ items: [], defaultKey: null });

  const add = [...target.querySelectorAll('button')].find((b) =>
    b.textContent.includes('添加启动项'),
  );
  add.click();
  await settle();

  expect(api.dirSize).toHaveBeenCalledWith('E:\\Games\\Foo');
  expect(target.querySelector('.path-field__suffix')?.textContent.trim()).toBe('1.0 GB');
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
