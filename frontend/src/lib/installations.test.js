import { expect, test } from 'vitest';
import {
  defaultKeyOf,
  draftSize,
  hasInstallation,
  installPathInvalid,
  joinLaunchPath,
  newInstallation,
  resolveDefaultId,
  splitLaunchPath,
  toInstallationDraft,
  toInstallationPayload,
} from './installations.js';

test('取回的行铺成一行启动路径', () => {
  const draft = toInstallationDraft({
    id: 'i1',
    game_id: 'g1',
    dir_path: 'D:\\Games\\One',
    entry_path: 'bin\\game.exe',
    size_bytes: 12,
    le_launch: true,
    magpie: 'window',
    launch_args: '-window',
  });

  expect(draft.id).toBe('i1');
  expect(draft.dirPath).toBe('D:\\Games\\One');
  expect(draft.entryPath).toBe('bin\\game.exe');
  expect(draft.path).toBe('D:\\Games\\One\\bin\\game.exe');
  expect(draft.sizeBytes).toBe(12);
  expect(draft.key).toMatch(
    /^local-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
  );
  expect(draft.leLaunch).toBe(true);
  expect(draft.magpie).toBe('window');
  expect(draft.launchArgs).toBe('-window');
});

test('只有目录的行铺成目录本身', () => {
  const draft = toInstallationDraft({ id: 'i1', dir_path: 'D:/Games/One' });
  expect(draft.path).toBe('D:/Games/One');
  expect(draft.entryPath).toBe('');
});

test('新建的安装项只有路径', () => {
  const draft = newInstallation('D:\\Games\\One\\game.exe');
  expect(draft.id).toBeNull();
  expect(draft.dirPath).toBe('');
  expect(draft.entryPath).toBe('');
  expect(draft.path).toBe('D:\\Games\\One\\game.exe');
  expect(draft.sizeBytes).toBeNull();
  expect(draft.leLaunch).toBe(false);
  expect(draft.magpie).toBe('off');
  expect(draft.launchArgs).toBe('');
});

test('只有已保存且路径未改的项才报出目录大小', () => {
  const saved = toInstallationDraft({
    id: 'i1',
    dir_path: 'D:\\Games\\One',
    entry_path: 'game.exe',
    size_bytes: 42,
  });
  expect(draftSize(saved)).toBe(42);
  expect(draftSize({ ...saved, path: 'D:\\Games\\Two\\game.exe' })).toBeNull();
  expect(draftSize(newInstallation('D:\\Games\\One\\game.exe'))).toBeNull();
});

test('拼接分隔符跟着目录走', () => {
  expect(joinLaunchPath('D:\\Games\\One', 'game.exe')).toBe('D:\\Games\\One\\game.exe');
  expect(joinLaunchPath('D:/Games/One', 'bin/game.exe')).toBe('D:/Games/One/bin/game.exe');
  expect(joinLaunchPath('D:\\Games\\One\\', 'game.exe')).toBe('D:\\Games\\One\\game.exe');
  expect(joinLaunchPath('D:\\Games\\One', '')).toBe('D:\\Games\\One');
});

test('拆出目录与启动文件', () => {
  expect(splitLaunchPath('D:\\Games\\One\\bin\\game.exe')).toEqual({
    dirPath: 'D:\\Games\\One\\bin',
    entryPath: 'game.exe',
  });
  expect(splitLaunchPath('D:/Games/One/game.exe')).toEqual({
    dirPath: 'D:/Games/One',
    entryPath: 'game.exe',
  });
  expect(splitLaunchPath('  D:\\Games\\One\\game.exe  ')).toEqual({
    dirPath: 'D:\\Games\\One',
    entryPath: 'game.exe',
  });
});

test('末段不是可执行文件时整串当目录', () => {
  expect(splitLaunchPath('D:\\Games\\One')).toEqual({ dirPath: 'D:\\Games\\One', entryPath: '' });
  expect(splitLaunchPath('D:\\Games\\One\\')).toEqual({ dirPath: 'D:\\Games\\One', entryPath: '' });
  expect(splitLaunchPath('D:\\Games\\0.0.1')).toEqual({ dirPath: 'D:\\Games\\0.0.1', entryPath: '' });
  expect(splitLaunchPath('D:\\Games\\One\\readme.txt')).toEqual({
    dirPath: 'D:\\Games\\One\\readme.txt',
    entryPath: '',
  });
});

test('可执行扩展名不分大小写', () => {
  expect(splitLaunchPath('D:\\Games\\One\\GAME.EXE')).toEqual({
    dirPath: 'D:\\Games\\One',
    entryPath: 'GAME.EXE',
  });
  expect(splitLaunchPath('D:\\Games\\One\\start.bat')).toEqual({
    dirPath: 'D:\\Games\\One',
    entryPath: 'start.bat',
  });
});

test('盘符根目录补齐分隔符', () => {
  expect(splitLaunchPath('D:\\game.exe')).toEqual({ dirPath: 'D:\\', entryPath: 'game.exe' });
});

test('没动过的行原样提交', () => {
  const nested = toInstallationDraft({
    id: 'i1',
    dir_path: 'D:\\Games\\One',
    entry_path: 'bin\\game.exe',
  });
  expect(toInstallationPayload(nested)).toEqual({
    id: 'i1',
    dir_path: 'D:\\Games\\One',
    entry_path: 'bin\\game.exe',
    le_launch: false,
    magpie: 'off',
    launch_args: null,
  });

  expect(toInstallationPayload({ ...nested, path: 'D:\\Games\\One\\bin\\other.exe' })).toEqual({
    id: 'i1',
    dir_path: 'D:\\Games\\One\\bin',
    entry_path: 'other.exe',
    le_launch: false,
    magpie: 'off',
    launch_args: null,
  });
});

test('提交时把路径拆成目录与启动文件', () => {
  const added = newInstallation('D:\\Games\\One\\bin\\game.exe');
  expect(toInstallationPayload(added)).toEqual({
    id: null,
    dir_path: 'D:\\Games\\One\\bin',
    entry_path: 'game.exe',
    le_launch: false,
    magpie: 'off',
    launch_args: null,
  });

  const kept = newInstallation('D:\\Games\\One\\launcher.exe');
  kept.id = 'i1';
  expect(toInstallationPayload(kept).id).toBe('i1');
});

test('启动方式开关与附加参数跟着提交', () => {
  const draft = {
    ...newInstallation('D:\\Games\\One\\game.exe'),
    leLaunch: true,
    magpie: 'window',
    launchArgs: '  -screen-fullscreen 0  ',
  };
  expect(toInstallationPayload(draft)).toEqual({
    id: null,
    dir_path: 'D:\\Games\\One',
    entry_path: 'game.exe',
    le_launch: true,
    magpie: 'window',
    launch_args: '-screen-fullscreen 0',
  });

  const cleared = { ...draft, launchArgs: '   ' };
  expect(toInstallationPayload(cleared).launch_args).toBeNull();

  const off = { ...draft, magpie: 'off' };
  expect(toInstallationPayload(off).magpie).toBe('off');
});

test('只有目录的行原样提交', () => {
  const draft = toInstallationDraft({ id: 'i1', dir_path: 'D:/Games/One' });
  expect(toInstallationPayload(draft)).toEqual({
    id: 'i1',
    dir_path: 'D:/Games/One',
    entry_path: null,
    le_launch: false,
    magpie: 'off',
    launch_args: null,
  });

  expect(toInstallationPayload({ ...draft, path: 'D:/Games/Two' })).toEqual({
    id: 'i1',
    dir_path: 'D:/Games/Two',
    entry_path: null,
    le_launch: false,
    magpie: 'off',
    launch_args: null,
  });
});

test('空路径与相对文件名都算没填全', () => {
  expect(installPathInvalid(newInstallation('   '))).toBe(true);
  expect(installPathInvalid(newInstallation('game.exe'))).toBe(true);
  expect(toInstallationPayload(newInstallation(''))).toBeNull();

  expect(installPathInvalid(newInstallation('D:\\Games\\One\\game.exe'))).toBe(false);
  expect(installPathInvalid(toInstallationDraft({ dir_path: 'D:/Games/One' }))).toBe(false);
});

test('默认项按库里的指定挑_没有就落到第一条', () => {
  const items = [{ key: 'k1', id: 'i1' }, { key: 'k2', id: 'i2' }];

  expect(defaultKeyOf(items, 'i2')).toBe('k2');
  expect(defaultKeyOf(items, null)).toBe('k1');
  expect(defaultKeyOf(items, 'i9')).toBe('k1');
  expect(defaultKeyOf([], 'i1')).toBeNull();
});

test('保存后按草稿下标记回默认项', () => {
  const items = [{ key: 'k1', id: 'i1' }, { key: 'k2', id: 'i2' }];
  const saved = [{ id: 'n1' }, { id: 'n2' }];

  expect(resolveDefaultId(saved, items, 'k2')).toBe('n2');
  expect(resolveDefaultId(saved, items, 'k1')).toBe('n1');
  expect(resolveDefaultId(saved, items, null)).toBe('n1');
  expect(resolveDefaultId(saved, items, 'gone')).toBe('n1');
  expect(resolveDefaultId([], [], null)).toBeNull();
});

test('识别已经加过的启动目标', () => {
  const items = [
    newInstallation('D:\\Games\\One\\game.exe'),
    newInstallation('D:\\Games\\Two'),
  ];
  expect(hasInstallation(items, newInstallation('D:\\Games\\One\\game.exe'))).toBe(true);
  expect(hasInstallation(items, newInstallation('D:\\Games\\Two\\'))).toBe(true);
  expect(hasInstallation(items, newInstallation('D:\\Games\\Three'))).toBe(false);
});

test('同一目录下的另一个启动文件不算重复', () => {
  const items = [newInstallation('D:\\Games\\One\\game.exe')];

  expect(hasInstallation(items, newInstallation('D:\\Games\\One\\game_zh.exe'))).toBe(false);
  expect(hasInstallation(items, newInstallation('D:\\Games\\Two\\game.exe'))).toBe(false);
});

test('分隔符不同但指向同一处算重复', () => {
  const items = [newInstallation('D:/Games/One/game.exe')];

  expect(hasInstallation(items, newInstallation('D:\\Games\\One\\game.exe'))).toBe(true);
});

test('同一行内重选启动文件不算重复', () => {
  const one = newInstallation('D:\\Games\\One\\game.exe');
  const items = [one, newInstallation('D:\\Games\\One\\other.exe')];

  expect(hasInstallation(items, newInstallation('D:\\Games\\One\\game.exe'), one.key)).toBe(false);
  expect(hasInstallation(items, newInstallation('D:\\Games\\One\\other.exe'), one.key)).toBe(true);
});

test('路径没填全时不参与判重', () => {
  const items = [newInstallation('D:\\Games\\One\\game.exe')];

  expect(hasInstallation(items, newInstallation(''))).toBe(false);
  expect(hasInstallation(items, newInstallation('game.exe'))).toBe(false);
});
