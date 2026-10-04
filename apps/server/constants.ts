/** 默认监听端口 */
export const PORT = 4545;

/** 默认监听地址：只给本机；要让局域网访问用 HOST=0.0.0.0 覆盖 */
export const DEFAULT_HOST = '127.0.0.1';

/** REST 接口前缀：静态托管要跳过它，否则 SPA 兜底会吞掉 API 路径 */
export const API_PREFIX = '/api';

/** 单文件数据库文件名，位于 DATA_DIR 下 */
export const DB_FILE = 'fishvn.db';