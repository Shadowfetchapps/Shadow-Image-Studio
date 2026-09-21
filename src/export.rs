use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use image::{DynamicImage, ExtendedColorType, ImageFormat};

use crate::document::ExportFormat;
use crate::error::{Error, Result};
use crate::paths;

pub fn export_copy(
    image: &DynamicImage,
    source: &Path,
    format: ExportFormat,
    quality: u8,
    strip_metadata: bool,
    dest_dir: Option<&Path>,
) -> Result<PathBuf> {
    let dir = dest_dir
        .map(|p| p.to_path_buf())
        .or_else(|| source.parent().map(|p| p.to_path_buf()))
        .ok_or_else(|| Error::user("Could not choose an export folder."))?;
    paths::ensure_dir(&dir)?;
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".into());
    let dest = paths::unique_path(&dir, &format!("{stem} (copy)"), format.ext());
    if dest.exists() {
        return Err(Error::user("Refusing to overwrite an existing file."));
    }
    write_image(image, &dest, format, quality)?;
    if strip_metadata {
        strip_file(&dest)?;
    }
    if !dest.is_file() || std::fs::metadata(&dest)?.len() == 0 {
        return Err(Error::user("The export produced an empty file."));
    }
    // Validate we can load it back.
    if format != ExportFormat::Avif {
        image::open(&dest).map_err(|err| {
            Error::detailed("The exported image could not be opened again.", err.to_string())
        })?;
    }
    let _ = source;
    Ok(dest)
}

pub fn encode_bytes(image: &DynamicImage, format: ExportFormat, quality: u8) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    match format {
        ExportFormat::Png => {
            image
                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
                .map_err(|e| Error::detailed("Could not encode PNG.", e.to_string()))?;
        }
        ExportFormat::Jpeg => {
            let rgb = image.to_rgb8();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
            encoder
                .encode(rgb.as_raw(), rgb.width(), rgb.height(), ExtendedColorType::Rgb8)
                .map_err(|e| Error::detailed("Could not encode JPEG.", e.to_string()))?;
        }
        ExportFormat::Webp => {
            image
                .write_to(&mut Cursor::new(&mut buf), ImageFormat::WebP)
                .map_err(|e| Error::detailed("Could not encode WebP.", e.to_string()))?;
        }
        ExportFormat::Tiff => {
            image
                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Tiff)
                .map_err(|e| Error::detailed("Could not encode TIFF.", e.to_string()))?;
        }
        ExportFormat::Avif => {
            return Err(Error::user(
                "AVIF size estimates use the encoder path; export writes the file with FFmpeg or ImageMagick.",
            ));
        }
    }
    Ok(buf)
}

fn write_image(image: &DynamicImage, dest: &Path, format: ExportFormat, quality: u8) -> Result<()> {
    match format {
        ExportFormat::Avif => write_avif(image, dest, quality),
        other => {
            let bytes = encode_bytes(image, other, quality)?;
            std::fs::write(dest, bytes)?;
            Ok(())
        }
    }
}

fn write_avif(image: &DynamicImage, dest: &Path, quality: u8) -> Result<()> {
    let tmp = dest.with_extension("png.tmp-studio");
    image
        .save(&tmp)
        .map_err(|e| Error::detailed("Could not stage the image for AVIF.", e.to_string()))?;
    if which::which("convert").is_ok() {
        let status = Command::new("convert")
            .arg(&tmp)
            .arg("-quality")
            .arg(quality.to_string())
            .arg(dest)
            .status();
        let _ = std::fs::remove_file(&tmp);
        if let Ok(st) = status {
            if st.success() && dest.is_file() {
                return Ok(());
            }
        }
    }
    let status = Command::new("ffmpeg")
        .args(["-hide_banner", "-y", "-loglevel", "error", "-i"])
        .arg(&tmp)
        .args(["-frames:v", "1", "-c:v", "libaom-av1", "-still-picture", "1"])
        .arg(dest)
        .status();
    let _ = std::fs::remove_file(&tmp);
    match status {
        Ok(st) if st.success() => Ok(()),
        Ok(_) => Err(Error::user(
            "Could not write AVIF. Install ImageMagick with AVIF or FFmpeg with libaom-av1.",
        )),
        Err(err) => Err(Error::detailed("Could not start FFmpeg for AVIF.", err.to_string())),
    }
}

fn strip_file(path: &Path) -> Result<()> {
    if which::which("convert").is_ok() {
        let tmp = path.with_extension("stripped.tmp");
        let status = Command::new("convert")
            .arg(path)
            .arg("-strip")
            .arg(&tmp)
            .status()
            .map_err(|e| Error::detailed("Could not strip metadata.", e.to_string()))?;
        if status.success() && tmp.is_file() {
            std::fs::rename(&tmp, path)?;
            return Ok(());
        }
        let _ = std::fs::remove_file(tmp);
    }
    Ok(())
}
