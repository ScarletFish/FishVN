import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';

/**
 * 起一个跑在临时库上的真服。
 * DATA_DIR 在模块加载时求值，所以环境变量必须在 import server.js 之前设好。
 */
export async function startTestServer() {
  const dataDir = mkdtempSync(join(tmpdir(), 'fishvn-test-'));
  try {
    process.env.FISHVN_DATA_DIR = dataDir;

    const { startServer } = await import('../../../dist/server/server.js');
    const { closeDb } = await import('../../../dist/server/db.js');

    const running = await startServer(0, '127.0.0.1');

    return {
      base: `http://127.0.0.1:${running.port}`,
      async stop() {
        await running.close();
        closeDb();
        rmSync(dataDir, { recursive: true, force: true });
      },
    };
  } catch (err) {
    // 起服失败时调用方拿不到句柄，after 钩子也就不会跑，目录得在这里收
    rmSync(dataDir, { recursive: true, force: true });
    throw err;
  }
}

/**
 * 建一棵真实的目录树供文件系统相关用例使用。
 * makeDir 每次返回一个新目录：dir_path 唯一约束会让复用目录的用例互相干扰。
 */
export function makeFixture() {
  const root = mkdtempSync(join(tmpdir(), 'fishvn-fs-'));
  const library = join(root, 'library');
  mkdirSync(library, { recursive: true });

  let seq = 0;
  const makeDir = (files) => {
    const dir = join(library, `case-${++seq}`);
    for (const [rel, content] of Object.entries(files)) {
      const full = join(dir, rel);
      mkdirSync(dirname(full), { recursive: true });
      writeFileSync(full, content);
    }
    return dir;
  };

  return {
    library,
    makeDir,
    cleanup: () => rmSync(root, { recursive: true, force: true }),
  };
}