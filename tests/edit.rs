use std::path::PathBuf;

use image::{DynamicImage, ImageBuffer, Rgba};
use shadow_image_studio::document::{CropRect, Document, ExportFormat};
use shadow_image_studio::export;

fn write_png(dir: &std::path::Path, name: &str, w: u32, h: u32) -> PathBuf {
    let path = dir.join(name);
    let img = DynamicImage::ImageRgba8(ImageBuffer::from_fn(w, h, |x, y| {
        Rgba([x as u8, y as u8, 80, 255])
    }));
    img.save(&path).unwrap();
    path
}

#[test]
fn rotate_flip_resize_export() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "src.png", 40, 20);
    let original = std::fs::read(&src).unwrap();
    let mut doc = Document::open(&src).unwrap();
    assert_eq!(doc.dimensions(), (40, 20));
    doc.rotate_cw();
    assert_eq!(doc.dimensions(), (20, 40));
    doc.flip_h();
    doc.resize(10, 20).unwrap();
    assert_eq!(doc.dimensions(), (10, 20));
    let jpeg = export::export_copy(
        &doc.image,
        &src,
        ExportFormat::Jpeg,
        70,
        false,
        Some(dir.path()),
    )
    .unwrap();
    let webp = export::export_copy(
        &doc.image,
        &src,
        ExportFormat::Webp,
        80,
        false,
        Some(dir.path()),
    )
    .unwrap();
    assert!(jpeg.exists());
    assert!(webp.exists());
    assert_ne!(jpeg, src);
    assert_eq!(std::fs::read(&src).unwrap(), original);
    image::open(&jpeg).unwrap();
}

#[test]
fn quality_changes_jpeg_size() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "q.png", 80, 80);
    let doc = Document::open(&src).unwrap();
    let small = export::encode_bytes(&doc.image, ExportFormat::Jpeg, 40).unwrap();
    let big = export::encode_bytes(&doc.image, ExportFormat::Jpeg, 95).unwrap();
    assert!(big.len() > small.len());
}

#[test]
fn crop_undo_redo() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "c.png", 30, 30);
    let mut doc = Document::open(&src).unwrap();
    doc.crop(CropRect {
        x: 5,
        y: 5,
        w: 10,
        h: 10,
    })
    .unwrap();
    assert_eq!(doc.dimensions(), (10, 10));
    assert!(doc.undo());
    assert_eq!(doc.dimensions(), (30, 30));
}

#[test]
fn unicode_name_and_no_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let src = write_png(dir.path(), "café 🖼️.png", 16, 16);
    let doc = Document::open(&src).unwrap();
    let a = export::export_copy(&doc.image, &src, ExportFormat::Png, 90, false, Some(dir.path()))
        .unwrap();
    let b = export::export_copy(&doc.image, &src, ExportFormat::Png, 90, false, Some(dir.path()))
        .unwrap();
    assert_ne!(a, b);
    assert!(src.exists());
}
