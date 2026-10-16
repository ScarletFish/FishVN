import { beforeEach, expect, test, vi } from 'vitest';

const { thumbnail, allowLocalFile, convertFileSrc } = vi.hoisted(() => ({
  thumbnail: vi.fn(),
  allowLocalFile: vi.fn(),
  convertFileSrc: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path) => convertFileSrc(path),
}));

vi.mock('./api.js', () => ({
  api: {
    thumbnail: (path) => thumbnail(path),
    allowLocalFile: (path) => allowLocalFile(path),
  },
}));

/** cache 是模块级状态，每条用例重新 import 拿干净的一份 */
async function load() {
  vi.resetModules();
  return import('./cover-src.js');
}

beforeEach(() => {
  vi.clearAllMocks();
  thumbnail.mockImplementation((path) => Promise.resolve(`${path}.thumb.jpg`));
  allowLocalFile.mockResolvedValue(undefined);
  convertFileSrc.mockImplementation((path) => `asset://localhost/${path}`);
});

test('交给 convertFileSrc 的是缩图路径，不是原图路径', async () => {
  const { coverSrc } = await load();

  const url = await coverSrc('D:\\pics\\mine.jpg');

  expect(thumbnail).toHaveBeenCalledWith('D:\\pics\\mine.jpg');
  expect(allowLocalFile).toHaveBeenCalledWith('D:\\pics\\mine.jpg.thumb.jpg');
  expect(url).toBe('asset://localhost/D:\\pics\\mine.jpg.thumb.jpg');
});

test('第二次调用命中缓存，不再请求缩图', async () => {
  const { coverSrc } = await load();

  const first = await coverSrc('p.jpg');
  const second = await coverSrc('p.jpg');

  expect(second).toBe(first);
  expect(thumbnail).toHaveBeenCalledTimes(1);
});

test('同一路径并发调用只请求一次', async () => {
  const { coverSrc } = await load();

  const [a, b] = await Promise.all([coverSrc('p.jpg'), coverSrc('p.jpg')]);

  expect(a).toBe(b);
  expect(thumbnail).toHaveBeenCalledTimes(1);
});

test('空路径不请求也不进缓存', async () => {
  const { coverSrc } = await load();

  expect(await coverSrc('')).toBeNull();
  expect(await coverSrc(null)).toBeNull();
  expect(thumbnail).not.toHaveBeenCalled();
});
