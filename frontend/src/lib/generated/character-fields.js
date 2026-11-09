export const CHARACTER_FIELDS = [
  { key: "game_id", sql: "game_id", draft: null, ty: "text", same: null, numeric: null, identity: true, joined: false, inDdl: true, inColumns: false, inStruct: false, inDraft: false, inMerge: false },
  { key: "id", sql: "id", draft: "id", ty: "text", same: null, numeric: null, identity: true, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: false },
  { key: "name", sql: "name", draft: "name", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "original", sql: "original", draft: "original", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "aliases", sql: "aliases", draft: "aliases", ty: "strings", same: "list", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "description", sql: "description", draft: "description", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "gender", sql: "gender", draft: "gender", ty: "strings", same: "list", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "age", sql: "age", draft: "age", ty: "integer", same: "number", numeric: {"max":99,"maxIntDigits":2}, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "birthday", sql: "birthday", draft: "birthday", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "blood_type", sql: "blood_type", draft: "bloodType", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "height", sql: "height", draft: "height", ty: "integer", same: "number", numeric: {"max":300,"maxIntDigits":3}, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "weight", sql: "weight", draft: "weight", ty: "integer", same: "number", numeric: {"max":300,"maxIntDigits":3}, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "bust", sql: "bust", draft: "bust", ty: "integer", same: "number", numeric: {"max":300,"maxIntDigits":3}, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "waist", sql: "waist", draft: "waist", ty: "integer", same: "number", numeric: {"max":300,"maxIntDigits":3}, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "hips", sql: "hips", draft: "hips", ty: "integer", same: "number", numeric: {"max":300,"maxIntDigits":3}, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "cup", sql: "cup", draft: "cup", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "role", sql: "role", draft: "role", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "image_id", sql: "image_id", draft: null, ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: false, inMerge: true },
  { key: "image_path", sql: "image_path", draft: "imagePath", ty: "text", same: "text", numeric: null, identity: false, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: true, inMerge: true },
  { key: "sort_order", sql: "sort_order", draft: null, ty: "integer", same: null, numeric: null, identity: true, joined: false, inDdl: true, inColumns: true, inStruct: true, inDraft: false, inMerge: false },
  { key: "traits", sql: "traits", draft: "traits", ty: "join", same: "listTrait", numeric: null, identity: false, joined: true, inDdl: false, inColumns: false, inStruct: true, inDraft: true, inMerge: true },
  { key: "va", sql: "va", draft: "va", ty: "join", same: "listVa", numeric: null, identity: false, joined: true, inDdl: false, inColumns: false, inStruct: true, inDraft: true, inMerge: true },
];

export const CHARACTER_DRAFT_META = ["source"];

export const CHARACTER_PAYLOAD_KEYS = CHARACTER_FIELDS.filter((f) => f.inDraft).map((f) => f.key);

export const CHARACTER_DRAFT_KEYS = [
  ...CHARACTER_FIELDS.filter((f) => f.inDraft).map((f) => f.draft),
  ...CHARACTER_DRAFT_META,
];

export const CHARACTER_MERGE_FIELDS = CHARACTER_FIELDS.filter((f) => f.inMerge);

export const CHARACTER_MERGE_KEYS = CHARACTER_MERGE_FIELDS.map((f) => f.key);

export const CHARACTER_NUMERIC_FIELDS = Object.fromEntries(
  CHARACTER_FIELDS.filter((f) => f.numeric).map((f) => [f.key, f.numeric]),
);
