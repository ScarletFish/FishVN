use serde::{Deserialize, Serialize};

use crate::types::{CharacterTrait, GameCharacter, MetadataStaff};

#[derive(Deserialize)]
pub struct VnEnvelope<T> {
    pub results: Vec<T>,
    pub more: bool,
    pub count: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct UlistVn {
    pub title: Option<String>,
    pub alttitle: Option<String>,
    pub olang: Option<String>,
    pub titles: Option<Vec<VnTitle>>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct UlistLabel {
    pub id: i64,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct UlistEntry {
    pub id: String,
    pub vn: UlistVn,
    pub labels: Vec<UlistLabel>,
    pub vote: Option<i64>,
    pub lastmod: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LabelList {
    pub labels: Vec<UlistLabel>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AuthInfo {
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnImage {
    pub id: Option<String>,
    pub url: Option<String>,
    pub thumbnail: Option<String>,
    pub dims: Option<Vec<i64>>,
    pub thumbnail_dims: Option<Vec<i64>>,
    #[serde(skip_deserializing)]
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnSearchHit {
    pub id: String,
    pub title: String,
    pub alttitle: Option<String>,
    pub released: Option<String>,
    pub olang: Option<String>,
    pub image: Option<VnImage>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnTitle {
    pub lang: String,
    pub title: Option<String>,
    pub latin: Option<String>,
    pub official: Option<bool>,
    pub main: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnTag {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub rating: Option<f64>,
    pub spoiler: Option<i64>,
    pub lie: Option<bool>,
    pub vn_count: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnDeveloper {
    pub id: String,
    pub name: Option<String>,
    pub original: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnMetadata {
    pub id: String,
    pub title: String,
    pub alttitle: Option<String>,
    pub olang: Option<String>,
    pub released: Option<String>,
    pub devstatus: Option<i64>,
    pub rating: Option<f64>,
    pub votecount: Option<i64>,
    pub length_minutes: Option<i64>,
    pub description: Option<String>,
    pub titles: Option<Vec<VnTitle>>,
    pub image: Option<VnImage>,
    pub tags: Option<Vec<VnTag>>,
    pub developers: Option<Vec<VnDeveloper>>,
    pub staff: Option<Vec<MetadataStaff>>,
    pub va: Option<Vec<VnVa>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnVa {
    pub note: Option<String>,
    pub staff: Option<VnVaStaff>,
    pub character: Option<VnVaCharacter>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnVaStaff {
    pub id: String,
    pub name: Option<String>,
    pub original: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VnVaCharacter {
    pub id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct VnSync {
    #[serde(flatten)]
    pub vn: VnMetadata,
    pub characters: Vec<GameCharacter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageKind {
    Cover,
    Character { character_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageJob {
    pub game_id: String,
    pub kind: ImageKind,
    pub image_id: String,
    pub url: String,
    pub thumb: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct CharRow {
    pub id: String,
    pub name: Option<String>,
    pub original: Option<String>,
    pub aliases: Option<Vec<Option<String>>>,
    pub description: Option<String>,
    pub gender: Option<Vec<Option<String>>>,
    pub age: Option<i64>,
    pub birthday: Option<Vec<Option<i64>>>,
    pub blood_type: Option<String>,
    pub height: Option<i64>,
    pub weight: Option<i64>,
    pub bust: Option<i64>,
    pub waist: Option<i64>,
    pub hips: Option<i64>,
    pub cup: Option<String>,
    pub image: Option<CharImage>,
    pub traits: Option<Vec<CharacterTrait>>,
    pub vns: Option<Vec<CharVn>>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct CharImage {
    pub id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct CharVn {
    pub id: Option<String>,
    pub role: Option<String>,
}
