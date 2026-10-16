//! 不发真实网络请求：CI 无网络会挂。反序列化用例锁住我们与真实响应的契约，
//! JSON 片段用 META_FIELDS / SEARCH_FIELDS 向 api.vndb.org 实抓后裁剪而成。

mod common;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use fishvn::vndb::{
    take_slot, validate_keyword, validate_vn_id, RateLimiter, VnEnvelope, VnMetadata,
};

#[test]
fn 满员时给出等到最早一次出窗的时间() {
    let base = Instant::now();
    let window = Duration::from_secs(10);
    let mut history = VecDeque::from([base]);

    let wait = take_slot(&mut history, 1, window, base + Duration::from_secs(4));

    assert_eq!(wait.unwrap_err(), Duration::from_secs(6));
}

#[test]
fn 出窗边界上的等待被抬到最少_1ms() {
    let base = Instant::now();
    let window = Duration::from_millis(100);
    let mut history = VecDeque::from([base + Duration::from_nanos(1)]);

    let wait = take_slot(&mut history, 1, window, base + window);

    assert_eq!(wait.unwrap_err(), Duration::from_millis(1));
}

#[test]
fn 没满员时立刻放行并记账() {
    let mut history = VecDeque::new();

    assert!(take_slot(&mut history, 2, Duration::from_secs(60), Instant::now()).is_ok());
    assert_eq!(history.len(), 1);
}

#[test]
fn 正好出窗的记录被清掉() {
    let base = Instant::now();
    let window = Duration::from_secs(1);
    let mut history = VecDeque::from([base]);

    assert!(take_slot(&mut history, 1, window, base + window).is_ok());
}

#[tokio::test]
async fn 限流器满员后要等到窗口出位() {
    let limiter = RateLimiter::new_with(3, Duration::from_millis(200));

    let start = Instant::now();
    for _ in 0..4 {
        limiter.acquire().await;
    }
    assert!(
        start.elapsed() >= Duration::from_millis(200),
        "第四次 acquire 应等到最早那次出窗，实测 {:?}",
        start.elapsed()
    );
}

#[tokio::test]
async fn 限流器没满员时不等待() {
    let limiter = RateLimiter::new_with(5, Duration::from_secs(60));

    let start = Instant::now();
    for _ in 0..5 {
        limiter.acquire().await;
    }
    assert!(start.elapsed() < Duration::from_millis(200));
}

#[test]
fn 关键词去空白后不能为空() {
    assert_eq!(validate_keyword("  clannad  ").unwrap(), "clannad");
    for bad in ["", "   ", "\t\n"] {
        let err = validate_keyword(bad).unwrap_err();
        assert_eq!(common::variant(&err), "BadRequest", "输入 {bad:?}");
    }
}

#[test]
fn vndb_id_必须是_v_加数字() {
    assert_eq!(validate_vn_id(" v14 ").unwrap(), "v14");
    for bad in ["", "14", "v", "vx", "v1a"] {
        let err = validate_vn_id(bad).unwrap_err();
        assert_eq!(common::variant(&err), "BadRequest", "输入 {bad:?}");
    }
}

/// v30003 实测响应裁剪：rating 与 alttitle 为 null、votecount 3 印证
/// 「票数 < 5 时 rating 为 null」，developer 的 type 是 "in" 而非字面类别名
const META_JSON: &str = r#"{
  "more": false,
  "results": [
    {
      "id": "v30003",
      "title": "Last Rx.",
      "alttitle": null,
      "olang": "en",
      "released": "2020-12-11",
      "devstatus": 2,
      "rating": null,
      "votecount": 3,
      "description": "A mysterious guy with terminal illness.\n\n[From [url=https://nsaid.itch.io/last-rx]itch.io[/url]]",
      "titles": [
        { "lang": "en", "title": "Last Rx.", "latin": null, "official": true, "main": true }
      ],
      "image": {
        "dims": [256, 323],
        "id": "cv45134",
        "url": "https://t.vndb.org/cv/34/45134.jpg"
      },
      "tags": [
        {
          "category": "cont",
          "id": "g134",
          "lie": false,
          "name": "Female Protagonist",
          "rating": 2,
          "spoiler": 0,
          "vn_count": 14785
        },
        {
          "category": "tech",
          "id": "g235",
          "lie": false,
          "name": "No Sexual Content",
          "rating": 2,
          "spoiler": 0,
          "vn_count": 15048
        }
      ],
      "developers": [
        { "id": "p9180", "name": "NSAID", "original": null, "type": "in" }
      ]
    }
  ]
}"#;

#[test]
fn 元数据响应能反序列化_含_null_字段() {
    let env: VnEnvelope<VnMetadata> = serde_json::from_str(META_JSON).unwrap();

    assert!(!env.more);
    assert_eq!(env.count, None);
    assert_eq!(env.results.len(), 1);

    let vn = &env.results[0];
    assert_eq!(vn.id, "v30003");
    assert_eq!(vn.title, "Last Rx.");
    assert_eq!(vn.alttitle, None);
    assert_eq!(vn.olang.as_deref(), Some("en"));
    assert_eq!(vn.released.as_deref(), Some("2020-12-11"));
    assert_eq!(vn.devstatus, Some(2));
    assert_eq!(vn.rating, None);
    assert_eq!(vn.votecount, Some(3));
    assert!(vn.image.is_some());

    let titles = vn.titles.as_ref().unwrap();
    assert_eq!(titles.len(), 1);
    assert_eq!(titles[0].lang, "en");
    assert_eq!(titles[0].latin, None);
    assert_eq!(titles[0].official, Some(true));
    assert_eq!(titles[0].main, Some(true));

    let tags = vn.tags.as_ref().unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].id, "g134");
    assert_eq!(tags[0].name, "Female Protagonist");
    assert_eq!(tags[0].category.as_deref(), Some("cont"));
    assert_eq!(tags[0].spoiler, Some(0));
    assert_eq!(tags[0].vn_count, Some(14785));

    let devs = vn.developers.as_ref().unwrap();
    assert_eq!(devs[0].id, "p9180");
    assert_eq!(devs[0].name.as_deref(), Some("NSAID"));
    assert_eq!(devs[0].original, None);
    assert_eq!(devs[0].kind.as_deref(), Some("in"));
}

/// image 为 null 是实测存在的形状（样本 v25031），单独钉住
#[test]
fn 元数据响应能反序列化_封面为_null() {
    let env: VnEnvelope<VnMetadata> = serde_json::from_str(
        r#"{"more":false,"results":[{"id":"v25031","title":"t","image":null}]}"#,
    )
    .unwrap();
    assert!(env.results[0].image.is_none());
}

/// v30001 实测响应：搜索档只回 thumbnail，url / dims 恒为 None
const SEARCH_JSON: &str = r#"{
  "more": false,
  "results": [
    {
      "id": "v30001",
      "title": "Yiri Nüyou",
      "alttitle": "一日女友",
      "olang": "zh-Hans",
      "released": "2018-04-30",
      "image": {
        "id": "cv78491",
        "thumbnail": "https://t.vndb.org/cv.t/91/78491.jpg",
        "thumbnail_dims": [256, 144]
      }
    }
  ]
}"#;

#[test]
fn 搜索响应能反序列化_封面只带缩略图() {
    let env: VnEnvelope<fishvn::vndb::VnSearchHit> = serde_json::from_str(SEARCH_JSON).unwrap();

    assert!(!env.more);
    let hit = &env.results[0];
    assert_eq!(hit.id, "v30001");
    assert_eq!(hit.title, "Yiri Nüyou");
    assert_eq!(hit.alttitle.as_deref(), Some("一日女友"));
    assert_eq!(hit.released.as_deref(), Some("2018-04-30"));

    let image = hit.image.as_ref().unwrap();
    assert_eq!(image.url, None);
    assert_eq!(image.dims, None);
    assert_eq!(
        image.thumbnail.as_deref(),
        Some("https://t.vndb.org/cv.t/91/78491.jpg")
    );
    assert_eq!(image.thumbnail_dims, Some(vec![256, 144]));
}

#[test]
fn 缺失字段按_default_补空而不是报错() {
    let env: VnEnvelope<VnMetadata> =
        serde_json::from_str(r#"{"more":false,"results":[{"id":"v1","title":"t"}]}"#).unwrap();
    let vn = &env.results[0];
    assert_eq!(vn.title, "t");
    assert_eq!(vn.rating, None);
    assert!(vn.tags.is_none());
}
