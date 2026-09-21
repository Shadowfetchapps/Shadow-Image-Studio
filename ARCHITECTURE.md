# Architecture

The library (`document`, `export`, `metadata`) owns pixels and files. GTK only displays them.

- Load with the `image` crate, then apply EXIF orientation
- Edits push a 30-deep undo stack of rasters
- Export writes a unique `* (copy).*` next to the source unless a folder is chosen
- AVIF goes through ImageMagick or FFmpeg
