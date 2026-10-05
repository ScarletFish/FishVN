import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

/**
 * 起一个跑在临时库上的真服。
 * DATA_DIR 在模块加载时求值，所以环境变量必须在 import server.js 之前设好。
 */
export async function startTestServer() {
  const dataDir = mkdtempSync(join(tmpdir(), 'fishvn-test-'));
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
}