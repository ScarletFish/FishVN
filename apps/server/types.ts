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
  /** 启动这份作品时的默认安装 */
  default_installation_id: string | null;
  /** unix秒 */
  last_played_at: number | null;
  play_count: number;
}

/** installations 表行：一个本地目录 */
export interface Installation {
  id: string;
  /** 所属作品；null = 待认领 */
  game_id: string | null;
  dir_path: string;
  /** 入口 exe，相对 dir_path */
  entry_path: string | null;
  /** 候选入口清单 */
  exes: string[] | null;
  size_bytes: number | null;
}