use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::error::Result;

pub fn exif_orientation(path: &Path, _bytes: &[u8]) -> Option<u32> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    field.value.get_uint(0)
}

pub fn summarize(path: &Path) -> Result<Vec<(String, String)>> {
    let mut rows = Vec::new();
    if let Ok(meta) = std::fs::metadata(path) {
        rows.push(("File size".into(), format!("{} bytes", meta.len())));
    }
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);
        if let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) {
            for field in exif.fields() {
                if matches!(
                    field.tag,
                    exif::Tag::Make
                        | exif::Tag::Model
                        | exif::Tag::DateTime
                        | exif::Tag::DateTimeOriginal
                        | exif::Tag::Orientation
                        | exif::Tag::XResolution
                        | exif::Tag::PixelXDimension
                        | exif::Tag::PixelYDimension
                        | exif::Tag::FNumber
                        | exif::Tag::ExposureTime
                        | exif::Tag::ISOSpeed
                        | exif::Tag::FocalLength
                        | exif::Tag::Software
                ) {
                    rows.push((
                        field.tag.to_string(),
                        field.display_value().with_unit(&exif).to_string(),
                    ));
                }
            }
        }
    }
    if rows.len() <= 1 {
        rows.push((
            "EXIF".into(),
            "No camera metadata found (common for PNG/WebP).".into(),
        ));
    }
    Ok(rows)
}
