# Shadow Image Studio

A fast, lightweight native image editor for Linux. Crop, resize, rotate, flip, adjust, inspect metadata, and export copies. It is not GIMP and not a web app.

![Icon](data/icons/hicolor/128x128/apps/shadow-image-studio.png)

## Features

- Open or drop PNG, JPEG, WebP, TIFF, and other formats the `image` crate can decode
- Large preview with crop handles (free, 1:1, 4:3, 16:9, 3:2)
- Resize in pixels or presets, with optional aspect lock
- Brightness, contrast, saturation, exposure, sharpness + reset
- Undo / redo
- Export Save Copy by default (PNG / JPEG / WebP / AVIF / TIFF)
- Quality slider with estimated size
- EXIF summary and optional metadata strip
- Shortcuts: Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+Z, Ctrl+Shift+Z
- Remembers export settings locally

## Screenshots

See `docs/screenshots/` (reserved if empty).

## Limitations

- Not a layered compositor. One image, one history stack (30 steps)
- AVIF export needs ImageMagick AVIF or FFmpeg `libaom-av1`
- Live adjustment sliders apply when you press **Apply adjustments**
- Color management is practical, not a full CMS; sRGB-ish pixel math

## License

MIT
