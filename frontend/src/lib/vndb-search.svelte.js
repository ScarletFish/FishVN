import { api } from './api.js';
import { readCache, writeCache } from './vndb-cache.js';

/** VNDB 搜索侧栏的状态与动作。侧栏内容由 Modal 的 aside 渲染，
 *  关闭时那一段没有挂载，父组件的工具栏按钮因而调不到这里的东西 */
export function createVndbSearch() {
  let searching = $state(false);
  let searchError = $state('');
  let hits = $state([]);

  async function search(keyword) {
    const term = keyword.trim();
    if (!term || searching) return;

    searchError = '';

    const cached = readCache(term);
    if (cached) {
      hits = cached;
      searching = false;
      return;
    }

    // 换词重搜时先清空，避免旧结果与新请求的空窗期显示错配
    hits = [];
    searching = true;
    try {
      const result = await api.searchVndb(term);
      writeCache(term, result);
      hits = result;
    } catch (err) {
      // 后端与网络错误不走 i18n，原样显示
      searchError = err.message;
    } finally {
      searching = false;
    }
  }

  /** 表单每次打开都要清一遍 */
  function reset() {
    searching = false;
    searchError = '';
    hits = [];
  }

  // 返回 getter：直接 return $state 变量只会把初始值交出去，调用方读到的永远是空
  return {
    get searching() {
      return searching;
    },
    get searchError() {
      return searchError;
    },
    get hits() {
      return hits;
    },
    search,
    reset,
  };
}