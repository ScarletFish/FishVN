#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CharacterScope {
    #[default]
    #[serde(rename = "main_only")]
    MainOnly,
    #[serde(rename = "with_side")]
    WithSide,
}

#[derive(Debug, Clone, Copy)]
pub struct RoleSpec {
    pub id: &'static str,
    pub tier: usize,
    pub min_scope: Option<CharacterScope>,
}

pub const ROLE_SPECS: &[RoleSpec] = &[
    RoleSpec { id: "main", tier: 0, min_scope: Some(CharacterScope::MainOnly) },
    RoleSpec { id: "primary", tier: 1, min_scope: Some(CharacterScope::MainOnly) },
    RoleSpec { id: "side", tier: 2, min_scope: Some(CharacterScope::WithSide) },
    RoleSpec { id: "appears", tier: 3, min_scope: None },
];

pub const UNKNOWN_ROLE_TIER: usize = 4;

impl CharacterScope {
    pub fn rank(self) -> u8 {
        match self {
            Self::MainOnly => 0,
            Self::WithSide => 1,
        }
    }

    pub fn allows(self, role: &str) -> bool {
        ROLE_SPECS
            .iter()
            .find(|spec| spec.id == role)
            .and_then(|spec| spec.min_scope)
            .is_some_and(|min| min.rank() <= self.rank())
    }
}

pub fn role_tier(role: Option<&str>) -> usize {
    role.and_then(|id| ROLE_SPECS.iter().find(|spec| spec.id == id).map(|spec| spec.tier))
        .unwrap_or(UNKNOWN_ROLE_TIER)
}
