# Building

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev librsvg2-bin desktop-file-utils ffmpeg imagemagick
cargo test --lib --tests
cargo build --release
./scripts/install-user.sh
./scripts/build-deb.sh
```
