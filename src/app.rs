use std::process::ExitCode;

use gtk::gio;
use gtk::prelude::*;

use shadow_image_studio::paths;

pub fn run() -> ExitCode {
    adw::init().expect("libadwaita init");
    let app = adw::Application::builder()
        .application_id(paths::APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    app.connect_startup(|_| crate::ui::register_icons());
    app.connect_activate(|app| {
        if app.active_window().is_none() {
            crate::ui::window::present(app, None);
        } else if let Some(win) = app.active_window() {
            win.present();
        }
    });
    app.connect_open(|app, files, _| {
        let path = files.first().and_then(|f| f.path());
        crate::ui::window::present(app, path);
    });
    let code = app.run();
    if code == gtk::glib::ExitCode::SUCCESS {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
