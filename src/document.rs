use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, RgbaImage};

use crate::error::{Error, Result};
use crate::metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Jpeg,
    Webp,
    Avif,
    Tiff,
}

impl ExportFormat {
    pub fn ext(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Avif => "avif",
            Self::Tiff => "tiff",
        }
    }

    pub fn from_index(i: u32) -> Self {
        match i {
            1 => Self::Jpeg,
            2 => Self::Webp,
            3 => Self::Avif,
            4 => Self::Tiff,
            _ => Self::Png,
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Png => 0,
            Self::Jpeg => 1,
            Self::Webp => 2,
            Self::Avif => 3,
            Self::Tiff => 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Adjust {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub exposure: f32,
    pub sharpness: f32,
}

impl Adjust {
    pub fn is_identity(self) -> bool {
        self.brightness.abs() < 0.001
            && self.contrast.abs() < 0.001
            && self.saturation.abs() < 0.001
            && self.exposure.abs() < 0.001
            && self.sharpness.abs() < 0.001
    }
}

pub struct Document {
    pub source: PathBuf,
    pub image: DynamicImage,
    pub orientation_applied: bool,
    undo: Vec<DynamicImage>,
    redo: Vec<DynamicImage>,
}

impl Document {
    pub fn open(path: &Path) -> Result<Self> {
        if path.is_dir() {
            return Err(Error::user("That is a folder. Open an image file."));
        }
        let meta = std::fs::metadata(path)?;
        if meta.len() == 0 {
            return Err(Error::user("This file is empty."));
        }
        let bytes = std::fs::read(path)?;
        let mut reader = ImageReader::new(Cursor::new(bytes.clone()))
            .with_guessed_format()
            .map_err(|err| Error::detailed("Could not read this image.", err.to_string()))?;
        reader.no_limits();
        let mut image = reader
            .decode()
            .map_err(|err| Error::detailed("This file is not a supported image.", err.to_string()))?;
        let orientation = metadata::exif_orientation(path, &bytes);
        let applied = if let Some(o) = orientation {
            image = apply_orientation(image, o);
            true
        } else {
            false
        };
        Ok(Self {
            source: path.to_path_buf(),
            image,
            orientation_applied: applied,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    fn push_undo(&mut self) {
        self.undo.push(self.image.clone());
        if self.undo.len() > 30 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(self.image.clone());
            self.image = prev;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.image.clone());
            self.image = next;
            true
        } else {
            false
        }
    }

    pub fn rotate_cw(&mut self) {
        self.push_undo();
        self.image = self.image.rotate90();
    }

    pub fn rotate_ccw(&mut self) {
        self.push_undo();
        self.image = self.image.rotate270();
    }

    pub fn flip_h(&mut self) {
        self.push_undo();
        self.image = self.image.fliph();
    }

    pub fn flip_v(&mut self) {
        self.push_undo();
        self.image = self.image.flipv();
    }

    pub fn crop(&mut self, rect: CropRect) -> Result<()> {
        let (iw, ih) = self.image.dimensions();
        if rect.w < 1 || rect.h < 1 || rect.x + rect.w > iw || rect.y + rect.h > ih {
            return Err(Error::user("The crop rectangle is outside the image."));
        }
        self.push_undo();
        self.image = self.image.crop_imm(rect.x, rect.y, rect.w, rect.h);
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width < 1 || height < 1 {
            return Err(Error::user("Width and height must be at least 1 pixel."));
        }
        self.push_undo();
        self.image = self.image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        Ok(())
    }

    pub fn apply_adjust(&mut self, adj: Adjust) {
        if adj.is_identity() {
            return;
        }
        self.push_undo();
        self.image = adjust_image(&self.image, adj);
    }

    pub fn rgba8(&self) -> RgbaImage {
        self.image.to_rgba8()
    }
}

fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

fn adjust_image(src: &DynamicImage, adj: Adjust) -> DynamicImage {
    let mut rgba = src.to_rgba8();
    let contrast = 1.0 + adj.contrast;
    let exposure = 2f32.powf(adj.exposure);
    let sat = 1.0 + adj.saturation;
    for px in rgba.pixels_mut() {
        let mut r = px[0] as f32 / 255.0;
        let mut g = px[1] as f32 / 255.0;
        let mut b = px[2] as f32 / 255.0;
        r = ((r - 0.5) * contrast + 0.5 + adj.brightness) * exposure;
        g = ((g - 0.5) * contrast + 0.5 + adj.brightness) * exposure;
        b = ((b - 0.5) * contrast + 0.5 + adj.brightness) * exposure;
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        r = luma + (r - luma) * sat;
        g = luma + (g - luma) * sat;
        b = luma + (b - luma) * sat;
        px[0] = (r.clamp(0.0, 1.0) * 255.0) as u8;
        px[1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
        px[2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
    }
    let mut out = DynamicImage::ImageRgba8(rgba);
    if adj.sharpness.abs() > 0.01 {
        let amount = (0.4 + adj.sharpness.abs() * 1.2).clamp(0.2, 2.2);
        out = DynamicImage::ImageRgba8(image::imageops::unsharpen(&out.to_rgba8(), amount, 2));
    }
    out
}

pub fn encode_estimate(img: &DynamicImage, format: ExportFormat, quality: u8) -> Result<u64> {
    if format == ExportFormat::Avif {
        let (w, h) = img.dimensions();
        return Ok((w as u64 * h as u64 * quality as u64) / 90);
    }
    crate::export::encode_bytes(img, format, quality).map(|b| b.len() as u64)
}

pub fn image_format_hint(path: &Path) -> Option<ImageFormat> {
    ImageFormat::from_path(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn solid() -> DynamicImage {
        DynamicImage::ImageRgba8(ImageBuffer::from_pixel(20, 10, Rgba([10, 20, 30, 255])))
    }

    #[test]
    fn crop_and_undo() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.png");
        solid().save(&path).unwrap();
        let mut doc = Document::open(&path).unwrap();
        assert_eq!(doc.dimensions(), (20, 10));
        doc.crop(CropRect {
            x: 2,
            y: 1,
            w: 8,
            h: 6,
        })
        .unwrap();
        assert_eq!(doc.dimensions(), (8, 6));
        assert!(doc.undo());
        assert_eq!(doc.dimensions(), (20, 10));
        assert!(doc.redo());
        assert_eq!(doc.dimensions(), (8, 6));
    }
}
