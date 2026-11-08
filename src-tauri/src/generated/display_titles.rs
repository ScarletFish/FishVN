pub const ZH_LANGS: &[&str] = &["zh-Hans", "zh-Hant"];
pub const JA_LANGS: &[&str] = &["ja"];
pub const ZH_OFFICIAL_FIRST: bool = true;

#[derive(Debug, Clone, Default)]
pub struct TitleCandidate {
    pub lang: String,
    pub title: Option<String>,
    pub official: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct TitleInput {
    pub titles: Vec<TitleCandidate>,
    pub alttitle: Option<String>,
    pub olang: Option<String>,
    pub latin: String,
}

fn push_unique(out: &mut Vec<String>, raw: Option<&str>) {
    let text = raw.unwrap_or_default().trim().to_owned();
    if !text.is_empty() && !out.iter().any(|used| used == &text) {
        out.push(text);
    }
}

pub fn title_chain(input: &TitleInput) -> Vec<String> {
    let mut out = Vec::new();
    let zh: Vec<&TitleCandidate> = input.titles.iter().filter(|t| ZH_LANGS.contains(&t.lang.as_str())).collect();
    let preferred = if ZH_OFFICIAL_FIRST {
        zh.iter().find(|t| t.official != Some(false)).copied().or_else(|| zh.first().copied())
    } else {
        zh.first().copied()
    };
    if let Some(t) = preferred {
        push_unique(&mut out, t.title.as_deref());
    }
    for t in input.titles.iter().filter(|t| JA_LANGS.contains(&t.lang.as_str())) {
        push_unique(&mut out, t.title.as_deref());
    }
    if input.olang.as_deref() == Some("ja") {
        push_unique(&mut out, input.alttitle.as_deref());
    }
    push_unique(&mut out, Some(input.latin.as_str()));
    out
}
