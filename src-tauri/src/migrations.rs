use rusqlite_migration::{M, Migrations};

const MIGRATION_ARRAY: &[M<'_>] = &[
    M::up("SELECT 1"),
    M::up("UPDATE game_metadata SET showcase_mode = ambience_mode"),
    M::up(
        "UPDATE game_metadata SET showcase_focal_x = ambience_focal_x, showcase_focal_y = ambience_focal_y",
    ),
];

pub const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATION_ARRAY);
pub const LATEST_VERSION: usize = MIGRATION_ARRAY.len();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 迁移版本锁定() {
        assert_eq!(LATEST_VERSION, 3);
    }
}
