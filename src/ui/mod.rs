pub mod dialogs;
pub mod window;

use shadow_image_studio::paths;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub fn register_icons() {
    gtk::Window::set_default_icon_name(paths::APP_ICON);
    if let Some(display) = gtk::gdk::Display::default() {
        let theme = gtk::IconTheme::for_display(&display);
        if let Ok(cwd) = std::env::current_dir() {
            theme.add_search_path(cwd.join("data/icons"));
        }
    }
}

#[allow(dead_code)]
pub fn icon_paintable() -> Option<gtk::gdk::Texture> {
    const PNG: &[u8] =
        include_bytes!("../../data/icons/hicolor/256x256/apps/shadow-image-studio.png");
    gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from_static(PNG)).ok()
}

pub fn apply_theme(theme: shadow_image_studio::Theme) {
    adw::StyleManager::default().set_color_scheme(match theme {
        shadow_image_studio::Theme::System => adw::ColorScheme::Default,
        shadow_image_studio::Theme::Light => adw::ColorScheme::ForceLight,
        shadow_image_studio::Theme::Dark => adw::ColorScheme::ForceDark,
    });
}
