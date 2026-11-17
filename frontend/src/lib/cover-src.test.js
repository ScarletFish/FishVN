import { beforeEach, expect, test, vi } from 'vitest';

const { thumbnail, allowLocalFile, convertFileSrc, env } = vi.hoisted(() => ({
  thumbnail: vi.fn(),
  allowLocalFile: vi.fn(),
  convertFileSrc: vi.fn(),
  env: { DEV_HTTP: false },
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

vi.mock('./env.js', () => ({
  get DEV_HTTP() {
    return env.DEV_HTTP;
  },
  DEV_HTTP_BASE: '/dev-http',
}));

/** cache 是模块级状态，每条用例重新 import 拿干净的一份 */
async function load() {
  vi.resetModules();
  return import('./cover-src.js');
}

beforeEach(() => {
  vi.clearAllMocks();
  env.DEV_HTTP = false;
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

test('全尺寸取图跳过缩图，直接放行原路径', async () => {
  const { fullSrc } = await load();

  const url = await fullSrc('D:\\pics\\mine.jpg');

  expect(thumbnail).not.toHaveBeenCalled();
  expect(allowLocalFile).toHaveBeenCalledWith('D:\\pics\\mine.jpg');
  expect(url).toBe('asset://localhost/D:\\pics\\mine.jpg');
});

test('同一路径的缩图与全尺寸各走各的缓存', async () => {
  const { coverSrc, fullSrc } = await load();

  const small = await coverSrc('p.jpg');
  const full = await fullSrc('p.jpg');

  expect(small).toBe('asset://localhost/p.jpg.thumb.jpg');
  expect(full).toBe('asset://localhost/p.jpg');
  expect(thumbnail).toHaveBeenCalledTimes(1);
});

test('浏览器模式缩图走 dev-http 文件路由，路径经 URL 编码且不申请 asset 授权', async () => {
  env.DEV_HTTP = true;
  const { coverSrc } = await load();

  const url = await coverSrc('D:\\pics\\mine.jpg');

  expect(thumbnail).toHaveBeenCalledWith('D:\\pics\\mine.jpg');
  expect(allowLocalFile).not.toHaveBeenCalled();
  expect(convertFileSrc).not.toHaveBeenCalled();
  expect(url).toBe('/dev-http/file?path=D%3A%5Cpics%5Cmine.jpg.thumb.jpg');
});

test('浏览器模式全尺寸取图直接走原路径的文件路由', async () => {
  env.DEV_HTTP = true;
  const { fullSrc } = await load();

  const url = await fullSrc('D:\\pics\\mine.jpg');

  expect(thumbnail).not.toHaveBeenCalled();
  expect(allowLocalFile).not.toHaveBeenCalled();
  expect(convertFileSrc).not.toHaveBeenCalled();
  expect(url).toBe('/dev-http/file?path=D%3A%5Cpics%5Cmine.jpg');
});
