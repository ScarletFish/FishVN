use std::fs;
use std::path::Path;

use rusqlite::Connection;

use crate::error::Result;

/// 单文件数据库文件名，位于数据目录下
pub const DB_FILE: &str = "fishvn.db";

/// 打开数据目录下的库，目录不存在则先建
pub fn open(dir: &Path) -> Result<Connection> {
    fs::create_dir_all(dir)?;
    let conn = Connection::open(dir.join(DB_FILE))?;
    ensure_schema(&conn)?;
    Ok(conn)
}

/// 幂等：表结构演进都写在这里
pub fn ensure_schema(conn: &Connection) -> Result<()> {
    // 内建的 SQLite 编译时带 SQLITE_DEFAULT_FOREIGN_KEYS=1，外键默认就开着，这行看着多余。
    // 但建表全靠 ON DELETE CASCADE，去掉它等于把级联交给传递依赖的编译选项。
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(
        "
        -- 作品身份：显示名 + VNDB 绑定。路径、元数据、用户状态各在其表。
        -- vndb_id 可空且 UNIQUE：NULL 之间不算重复，一部 VN 只对应一条作品。
        CREATE TABLE IF NOT EXISTS games (
          id         TEXT PRIMARY KEY,
          name       TEXT NOT NULL,
          vndb_id    TEXT UNIQUE,
          created_at INTEGER NOT NULL
        );

        -- 用户状态：每部作品一行，随作品一同建立、随作品删除而清除。
        CREATE TABLE IF NOT EXISTS user_game_state (
          game_id        TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
          status         TEXT NOT NULL DEFAULT 'wishlist',
          user_rating    REAL,
          notes          TEXT,
          last_played_at INTEGER,
          play_count     INTEGER NOT NULL DEFAULT 0
        );

        -- 一个本地目录，可已认领为某作品也可待认领。路径唯一由文件管理器给出规范写法。
        CREATE TABLE IF NOT EXISTS installations (
          id         TEXT PRIMARY KEY,
          game_id    TEXT REFERENCES games(id) ON DELETE SET NULL,
          dir_path   TEXT NOT NULL UNIQUE,
          entry_path TEXT,
          exes       TEXT,
          size_bytes INTEGER
        );

        -- 作品资料：一部作品一行，每列都可空。作品正式名在 games.name，这里不重复存；
        -- VNDB 那份 titles 里已含 official / main 的正式标题，别名存 alias。
        -- titles / tags / developers 存 JSON 文本而非标量，要按其中某个键筛
        -- 只能整列取出后在 Rust 侧解。
        -- image_id 是 VNDB 侧的 cv 编号，本项目不写它：下载判重靠
        -- covers/vndb/<cv编号>.jpg 是否存在，同步冲突靠 image_path 是否为空。
        -- image_path 是本地文件的位置，两者不是一回事。
        CREATE TABLE IF NOT EXISTS game_metadata (
          game_id     TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
          alias       TEXT,
          titles      TEXT,
          rating      REAL,
          votecount   INTEGER,
          released    TEXT,
          description TEXT,
          image_id    TEXT,
          image_path  TEXT,
          tags        TEXT,
          developers  TEXT
        );

        -- 角色。属于作品，随作品删除而清除，不建跨作品的共享本体：
        -- VNDB 上同一角色在不同作品里就是不同条目（数据详略、立绘各自独立）。
        -- id 平时是 VNDB c 编号，用户手建的行是 local-<uuid>，两种混排。
        -- traits 是 trait 全量数组，一篇下来上百条，本地录入不给它入口；
        -- VNDB 同步时才写。
        CREATE TABLE IF NOT EXISTS characters (
          game_id     TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          id          TEXT NOT NULL,
          name        TEXT,
          original    TEXT,
          aliases     TEXT,
          description TEXT,
          gender      TEXT,
          age         INTEGER,
          birthday    TEXT,
          blood_type  TEXT,
          height      INTEGER,
          weight      INTEGER,
          bust        INTEGER,
          waist       INTEGER,
          hips        INTEGER,
          cup         TEXT,
          role        TEXT,
          image_id    TEXT,
          image_path  TEXT,
          sort_order  INTEGER NOT NULL DEFAULT 0,
          traits      TEXT,
          PRIMARY KEY (game_id, id)
        );

        -- 声优。实体表：换一部作品再遇到同一个 s 编号还是同一人。
        -- 本地手填的行没有 s 编号，用 local-<uuid> 占位；跨作品复用，行在无人引用时由下面的触发器回收
        CREATE TABLE IF NOT EXISTS staffs (
          id       TEXT PRIMARY KEY,
          name     TEXT,
          original TEXT
        );

        -- 配音关系。跟着角色行走，所以主键里带着 game_id。
        -- note 是版本标记，同一对角色与声优可有多条；SQLite 允许主键列为空，
        -- 空 note 一律存 ''，免得同一条关系被 NULL 与 '' 写成两行
        CREATE TABLE IF NOT EXISTS character_va (
          game_id      TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
          character_id TEXT NOT NULL,
          staff_id     TEXT NOT NULL REFERENCES staffs(id) ON DELETE CASCADE,
          note         TEXT NOT NULL DEFAULT '',
          PRIMARY KEY (game_id, character_id, staff_id, note),
          FOREIGN KEY (game_id, character_id) REFERENCES characters(game_id, id) ON DELETE CASCADE
        );

        -- 触发器的残留检查按 staff_id 查，主键前缀用不上，缺索引会全表扫
        CREATE INDEX IF NOT EXISTS character_va_staff ON character_va(staff_id);

        -- 声优的去留交给关系表判：角色删掉、连带删掉配音关系后，某个声优可能
        -- 一次关系都不剩，那行就再也走不到任何作品，是垃圾。
        -- 三条删除来源（删作品、删角色、整包替换）共用这一条回收路径
        CREATE TRIGGER IF NOT EXISTS character_va_prune_staff
        AFTER DELETE ON character_va
        BEGIN
          DELETE FROM staffs WHERE id = OLD.staff_id
            AND NOT EXISTS (
              SELECT 1 FROM character_va WHERE staff_id = OLD.staff_id
            );
        END;
        ",
    )?;

    // ADD COLUMN 不幂等，先查列在不在
    if !has_column(conn, "user_game_state", "default_installation_id")? {
        conn.execute_batch(
            "ALTER TABLE user_game_state ADD COLUMN default_installation_id TEXT
             REFERENCES installations(id) ON DELETE SET NULL",
        )?;
    }
    Ok(())
}

/// table 会拼进 SQL，只能传字面量，不要接调用方的变量
fn has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}
