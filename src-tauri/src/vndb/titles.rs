use super::dto::{UlistVn, VnMetadata, VnTitle};
use crate::generated::display_titles::{title_chain, TitleCandidate, TitleInput};

fn candidates(titles: Option<&[VnTitle]>) -> Vec<TitleCandidate> {
    titles
        .unwrap_or_default()
        .iter()
        .map(|t| TitleCandidate {
            lang: t.lang.clone(),
            title: t.title.clone(),
            official: t.official,
        })
        .collect()
}

pub fn chain(vn: &VnMetadata) -> Vec<String> {
    title_chain(&TitleInput {
        titles: candidates(vn.titles.as_deref()),
        alttitle: vn.alttitle.clone(),
        olang: vn.olang.clone(),
        latin: vn.title.clone(),
    })
}

pub fn ulist_chain(vn: &UlistVn) -> Vec<String> {
    title_chain(&TitleInput {
        titles: candidates(vn.titles.as_deref()),
        alttitle: vn.alttitle.clone(),
        olang: vn.olang.clone(),
        latin: vn.title.clone().unwrap_or_default(),
    })
}
