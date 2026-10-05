/** games 表行：作品身份 */
export interface Game {
  id: string;
  name: string;
  /** VNDB 编号 'v14'；null = 未匹配 */
  vndb_id: string | null;
  /** unix秒 */
  created_at: number;
}

/** 游玩状态，取值与 VNDB 的状态集一致 */
export const STATUSES = ['wishlist', 'playing', 'finished', 'dropped'] as const;

export type Status = (typeof STATUSES)[number];

/** user_game_state 表行 */
export interface UserGameState {
  game_id: string;
  status: Status;
  user_rating: number | null;
  notes: string | null;
  /** unix秒 */
  last_played_at: number | null;
  play_count: number;
}