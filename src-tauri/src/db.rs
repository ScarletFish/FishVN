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
        -- image_id 是 VNDB 侧的 cv 编号，只用来判断封面要不要重下；
        -- image_path 是本地缓存文件的位置，两者不是一回事。
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
