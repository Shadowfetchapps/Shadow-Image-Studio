use adw::prelude::*;
use gtk::glib;

use shadow_image_studio::paths;
use shadow_image_studio::settings::{Settings, Theme};
use shadow_image_studio::Error;

use crate::ui;

pub fn show_error(parent: &impl IsA<gtk::Window>, err: &Error) {
    let dialog = adw::MessageDialog::new(Some(parent), Some("Image problem"), Some(&err.human_message()));
    dialog.add_response("ok", "OK");
    if let Some(details) = err.technical_details() {
        dialog.add_response("details", "Show Technical Details");
        let parent = parent.as_ref().clone();
        dialog.connect_response(None, move |dlg, response| {
            if response == "details" {
                let tech = adw::MessageDialog::new(Some(&parent), Some("Technical details"), Some(&details));
                tech.add_response("ok", "OK");
                tech.present();
            }
            dlg.close();
        });
    }
    dialog.present();
}

pub fn show_about(parent: &impl IsA<gtk::Window>) {
    adw::AboutWindow::builder()
        .transient_for(parent)
        .modal(true)
        .application_name(paths::APP_NAME)
        .application_icon(paths::APP_ICON)
        .developer_name("Shadowfetch")
        .version(paths::APP_VERSION)
        .comments("A fast local image editor. Crop, resize, adjust, and export copies. Not GIMP.")
        .license_type(gtk::License::MitX11)
        .website(paths::APP_WEBSITE)
        .copyright("© 2026 Shadow Image Studio contributors")
        .build()
        .present();
}

pub fn show_settings(parent: &impl IsA<gtk::Window>, settings: &Settings, on_save: impl Fn(Settings) + 'static) {
    let window = adw::PreferencesWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("Settings")
        .search_enabled(false)
        .build();
    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    let theme = adw::ComboRow::new();
    theme.set_title("Theme");
    theme.set_model(Some(&gtk::StringList::new(&["System", "Light", "Dark"])));
    theme.set_selected(settings.theme.index());
    group.add(&theme);
    page.add(&group);
    window.add(&page);
    let current = std::rc::Rc::new(std::cell::RefCell::new(settings.clone()));
    let c = current.clone();
    theme.connect_selected_notify(move |row| {
        let t = Theme::from_index(row.selected());
        c.borrow_mut().theme = t;
        ui::apply_theme(t);
    });
    window.connect_close_request(move |_| {
        let snapshot = current.borrow().clone();
        let _ = snapshot.save();
        on_save(snapshot);
        glib::Propagation::Proceed
    });
    window.present();
}
