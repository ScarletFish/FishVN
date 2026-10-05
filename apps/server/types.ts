/** games 表行：作品身份 */
export interface Game {
  id: string;
  name: string;
  /** VNDB 编号 'v14'；null = 未匹配 */
  vndb_id: string | null;
  /** unix秒 */
  created_at: number;
}