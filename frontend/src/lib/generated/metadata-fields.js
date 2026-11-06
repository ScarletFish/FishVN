export const METADATA_FIELDS = [
  { key: "game_id", type: "text", blank: "not_applicable", label: null, source: "system", editable: false, clearable: false, inPatch: false },
  { key: "image_id", type: "text", blank: "not_applicable", label: null, source: "system", editable: false, clearable: false, inPatch: false },
  { key: "alias", type: "text", blank: "trim_empty", label: "别名", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "released", type: "text", blank: "trim_empty", label: "发售日", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "rating", type: "real", blank: "null_only", label: "评分", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "votecount", type: "integer", blank: "null_only", label: "评分人数", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "length_minutes", type: "integer", blank: "null_only", label: "时长", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "description", type: "text", blank: "trim_empty", label: "简介", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "image_path", type: "text", blank: "trim_empty", label: "封面", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "titles", type: "titles", blank: "empty_list", label: "标题", source: "upstream", editable: false, clearable: true, inPatch: true },
  { key: "tags", type: "tags", blank: "empty_list", label: "标签", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "developers", type: "strings", blank: "empty_list", label: "开发商", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "staff", type: "staff", blank: "empty_list", label: "制作组", source: "upstream", editable: true, clearable: true, inPatch: true },
  { key: "cover_focal_x", type: "integer", blank: "not_applicable", label: null, source: "local", editable: true, clearable: true, inPatch: true },
  { key: "cover_focal_y", type: "integer", blank: "not_applicable", label: null, source: "local", editable: true, clearable: true, inPatch: true },
  { key: "ambience_path", type: "text", blank: "not_applicable", label: null, source: "local", editable: true, clearable: true, inPatch: true },
  { key: "ambience_mode", type: "mode", blank: "not_applicable", label: null, source: "local", editable: true, clearable: false, inPatch: true },
  { key: "ambience_plate", type: "bool", blank: "not_applicable", label: null, source: "local", editable: true, clearable: false, inPatch: true },
  { key: "ambience_focal_x", type: "integer", blank: "not_applicable", label: null, source: "local", editable: true, clearable: true, inPatch: true },
  { key: "ambience_focal_y", type: "integer", blank: "not_applicable", label: null, source: "local", editable: true, clearable: true, inPatch: true },
];

export const PATCH_KEYS = METADATA_FIELDS.filter((field) => field.inPatch).map((field) => field.key);

export const FORM_KEYS = METADATA_FIELDS.filter((field) => field.inPatch && field.editable).map((field) => field.key);
