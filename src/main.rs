mod app;
mod ui;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "Shadow Image Studio — fast local image editor\n\n\
             Usage:\n\tshadow-image-studio [FILE]\n"
        );
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("shadow-image-studio {}", shadow_image_studio::paths::APP_VERSION);
        return ExitCode::SUCCESS;
    }
    app::run()
}
