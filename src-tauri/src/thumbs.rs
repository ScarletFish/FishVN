//! 封面派生图：把原图缩到长边 560 的 JPEG，缓存到 data_dir/covers/。
//!
//! 560 = --card-w(277px) 的 2 倍，覆盖到 200% 缩放屏，再大就是白 decode

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageReader};

use crate::error::{AppError, Result};

const MAX_EDGE: u32 = 560;

const QUALITY: u8 = 85;

pub fn cached(src: &str, data_dir: &Path) -> Result<PathBuf> {
    let src = std::path::PathBuf::from(src.trim());
    let meta =
        std::fs::metadata(&src).map_err(|_| AppError::BadRequest("封面文件不存在".into()))?;
    if !meta.is_file() {
        return Err(AppError::BadRequest("封面不是文件".into()));
    }

    let dir = data_dir.join("covers");
    let dst = dir.join(format!("{}.jpg", cache_key(&src, &meta)));
    if dst.is_file() {
        return Ok(dst);
    }
    std::fs::create_dir_all(&dir)?;
    generate(&src, &dst)?;
    Ok(dst)
}

/// 内容寻址：路径、size、mtime 变了就是新key，旧缓存自然失效，不必显式删。
/// mtime 精度到秒，所以 size 也参与哈希 —— 同秒内换成同尺寸的图仍会命中旧缓存
fn cache_key(src: &Path, meta: &std::fs::Metadata) -> u64 {
    let mut h = DefaultHasher::new();
    src.to_string_lossy().hash(&mut h);
    meta.len().hash(&mut h);
    meta.modified()
        .ok()
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        })
        .hash(&mut h);
    h.finish()
}

fn generate(src: &Path, dst: &Path) -> Result<()> {
    let img = decode(src)?;
    let small = fit(img, MAX_EDGE);
    let rgb = small.to_rgb8();
    let (w, h) = (rgb.width(), rgb.height());

    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, QUALITY)
        .write_image(rgb.as_raw(), w, h, ExtendedColorType::Rgb8)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    std::fs::write(dst, out)?;
    Ok(())
}

fn decode(src: &Path) -> Result<DynamicImage> {
    let reader =
        ImageReader::open(src).map_err(|e| AppError::BadRequest(format!("封面读不出来：{e}")))?;
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| AppError::BadRequest(format!("封面格式不支持：{e}")))?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)
        .map_err(|e| AppError::BadRequest(format!("封面解不开：{e}")))?;
    img.apply_orientation(orientation);
    Ok(img)
}

fn fit(img: DynamicImage, max_edge: u32) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    let long = w.max(h);
    if long <= max_edge {
        return img;
    }
    let scale = max_edge as f64 / long as f64;
    let (tw, th) = (
        (w as f64 * scale).round() as u32,
        (h as f64 * scale).round() as u32,
    );
    img.resize_exact(tw, th, image::imageops::FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn sample(dir: &Path, w: u32, h: u32) -> PathBuf {
        let p = dir.join(format!("s{w}x{h}.png"));
        let img = image::RgbImage::from_fn(w, h, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        let mut buf = Vec::new();
        image::codecs::png::PngEncoder::new(&mut buf)
            .write_image(img.as_raw(), w, h, image::ExtendedColorType::Rgb8)
            .unwrap();
        std::fs::File::create(&p).unwrap().write_all(&buf).unwrap();
        p
    }

    fn dims(p: &Path) -> (u32, u32) {
        image::open(p).map(|i| (i.width(), i.height())).unwrap()
    }

    fn sample_with_exif(dir: &Path, w: u32, h: u32, orientation: u16) -> PathBuf {
        let p = dir.join(format!("s{w}x{h}-o{orientation}.jpg"));
        let img = image::RgbImage::from_fn(w, h, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, QUALITY)
            .write_image(img.as_raw(), w, h, ExtendedColorType::Rgb8)
            .unwrap();

        let mut tiff = Vec::new();
        tiff.extend(b"II");
        tiff.extend([42u8, 0]);
        tiff.extend(8u32.to_le_bytes());
        tiff.extend(1u16.to_le_bytes());
        tiff.extend(0x0112u16.to_le_bytes());
        tiff.extend(3u16.to_le_bytes());
        tiff.extend(1u32.to_le_bytes());
        tiff.extend(orientation.to_le_bytes());
        tiff.extend(0u16.to_le_bytes());
        tiff.extend(0u32.to_le_bytes());

        let mut out = Vec::with_capacity(jpeg.len() + tiff.len() + 10);
        out.extend(&jpeg[..2]);
        out.extend([0xFF, 0xE1]);
        out.extend(((tiff.len() + 8) as u16).to_be_bytes());
        out.extend(b"Exif\0\0");
        out.extend(&tiff);
        out.extend(&jpeg[2..]);
        std::fs::File::create(&p).unwrap().write_all(&out).unwrap();
        p
    }

    #[test]
    fn 大图按比例缩到上限() {
        let tmp = tempfile::tempdir().unwrap();
        let src = sample(tmp.path(), 700, 980);
        let out = dir_dst(tmp.path());
        generate(&src, &out).unwrap();
        assert_eq!(dims(&out), (400, 560));
    }

    #[test]
    fn 小图不放大只换编码() {
        let tmp = tempfile::tempdir().unwrap();
        let src = sample(tmp.path(), 300, 400);
        let out = dir_dst(tmp.path());
        generate(&src, &out).unwrap();
        assert_eq!(dims(&out), (300, 400));
    }

    #[test]
    fn 横图按长边缩() {
        let tmp = tempfile::tempdir().unwrap();
        let src = sample(tmp.path(), 980, 700);
        let out = dir_dst(tmp.path());
        generate(&src, &out).unwrap();
        assert_eq!(dims(&out), (560, 400));
    }

    #[test]
    fn exif方向在缩放前应用_不然算错目标边长() {
        let tmp = tempfile::tempdir().unwrap();
        let src = sample_with_exif(tmp.path(), 700, 980, 6);
        let out = dir_dst(tmp.path());
        generate(&src, &out).unwrap();
        assert_eq!(dims(&out), (560, 400));
    }

    #[test]
    fn 二次调用命中缓存不改文件() {
        let tmp = tempfile::tempdir().unwrap();
        let src = sample(tmp.path(), 700, 980);
        let a = cached(src.to_str().unwrap(), tmp.path()).unwrap();
        let stamp = std::fs::metadata(&a).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let b = cached(src.to_str().unwrap(), tmp.path()).unwrap();
        assert_eq!(a, b);
        assert_eq!(stamp, std::fs::metadata(&b).unwrap().modified().unwrap());
    }

    #[test]
    fn 源图改动后换缓存() {
        let tmp = tempfile::tempdir().unwrap();
        let src = sample(tmp.path(), 700, 980);
        let a = cached(src.to_str().unwrap(), tmp.path()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        sample(tmp.path(), 700, 980);
        let b = cached(src.to_str().unwrap(), tmp.path()).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn 文件不存在报错() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(cached("nope.jpg", tmp.path()).is_err());
    }

    #[test]
    fn 不是文件报错() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(cached(tmp.path().to_str().unwrap(), tmp.path()).is_err());
    }

    #[test]
    fn 坏文件报错() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("bad.jpg");
        std::fs::write(&src, b"not an image").unwrap();
        assert!(cached(src.to_str().unwrap(), tmp.path()).is_err());
    }

    fn dir_dst(dir: &Path) -> PathBuf {
        dir.join("out.jpg")
    }
}
