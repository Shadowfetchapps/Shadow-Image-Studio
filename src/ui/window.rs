use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use gtk::gdk;
use gtk::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::gio;

use shadow_image_studio::document::{encode_estimate, Adjust, CropRect, Document, ExportFormat};
use shadow_image_studio::export;
use shadow_image_studio::metadata;
use shadow_image_studio::paths;
use shadow_image_studio::settings::Settings;
use shadow_image_studio::Error;

use crate::ui::{self, dialogs};

struct State {
    settings: RefCell<Settings>,
    doc: RefCell<Option<Document>>,
    crop: Cell<Option<CropRect>>,
    crop_ratio: Cell<f32>,
    drag: Cell<Option<u8>>,
}

struct Widgets {
    window: adw::ApplicationWindow,
    toast: adw::ToastOverlay,
    picture: gtk::Picture,
    area: gtk::DrawingArea,
    dim: gtk::Label,
    meta: gtk::Label,
    quality: gtk::Scale,
    format: gtk::DropDown,
    estimate: gtk::Label,
    strip: gtk::Switch,
    bright: gtk::Scale,
    contrast: gtk::Scale,
    sat: gtk::Scale,
    exposure: gtk::Scale,
    sharp: gtk::Scale,
    crop_on: gtk::ToggleButton,
    w_entry: gtk::Entry,
    h_entry: gtk::Entry,
    lock: gtk::CheckButton,
}

pub fn present(app: &adw::Application, initial: Option<PathBuf>) {
    ui::load_css();
    let settings = Settings::load();
    ui::apply_theme(settings.theme);
    if let Some(win) = app.active_window() {
        win.present();
        return;
    }

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(paths::APP_NAME)
        .default_width(1200)
        .default_height(840)
        .build();
    window.set_icon_name(Some(paths::APP_ICON));

    let toast = adw::ToastOverlay::new();
    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    let open = icon_btn("document-open-symbolic", "Open image (Ctrl+O)");
    let undo = icon_btn("edit-undo-symbolic", "Undo (Ctrl+Z)");
    let redo = icon_btn("edit-redo-symbolic", "Redo (Ctrl+Shift+Z)");
    let settings_btn = icon_btn("emblem-system-symbolic", "Settings");
    let about = icon_btn("help-about-symbolic", "About");
    header.pack_start(&open);
    header.pack_start(&undo);
    header.pack_start(&redo);
    header.pack_end(&settings_btn);
    header.pack_end(&about);
    toolbar.add_top_bar(&header);

    let picture = gtk::Picture::new();
    picture.set_can_shrink(true);
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    let area = gtk::DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.set_can_target(true);
    let overlay = gtk::Overlay::new();
    overlay.add_css_class("preview-frame");
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);
    overlay.set_child(Some(&picture));
    overlay.add_overlay(&area);
    let dim = gtk::Label::new(Some("Drop an image or press Ctrl+O"));
    dim.add_css_class("dim-label");
    dim.set_xalign(0.0);
    let preview = gtk::Box::new(gtk::Orientation::Vertical, 8);
    preview.set_hexpand(true);
    preview.append(&overlay);
    preview.append(&dim);

    let side = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let crop_on = gtk::ToggleButton::with_label("Crop mode");
    crop_on.set_tooltip_text(Some("Show drag handles"));
    let ratio = gtk::DropDown::from_strings(&["Free", "1:1", "4:3", "16:9", "3:2"]);
    let apply_crop = gtk::Button::with_label("Apply crop");
    side.append(&heading("Crop"));
    side.append(&crop_on);
    side.append(&ratio);
    side.append(&apply_crop);

    let w_entry = gtk::Entry::new();
    w_entry.set_placeholder_text(Some("Width px"));
    let h_entry = gtk::Entry::new();
    h_entry.set_placeholder_text(Some("Height px"));
    let lock = gtk::CheckButton::with_label("Keep aspect");
    lock.set_active(true);
    let rpresets = gtk::DropDown::from_strings(&["Presets…", "1920 wide", "1280 wide", "800 wide", "50%"]);
    let apply_resize = gtk::Button::with_label("Resize");
    side.append(&heading("Resize"));
    side.append(&w_entry);
    side.append(&h_entry);
    side.append(&lock);
    side.append(&rpresets);
    side.append(&apply_resize);

    let rot_l = gtk::Button::with_label("Rotate left");
    let rot_r = gtk::Button::with_label("Rotate right");
    let flip_h = gtk::Button::with_label("Flip H");
    let flip_v = gtk::Button::with_label("Flip V");
    side.append(&heading("Transform"));
    side.append(&rot_l);
    side.append(&rot_r);
    side.append(&flip_h);
    side.append(&flip_v);

    let bright = labeled_scale("Brightness", -0.6, 0.6);
    let contrast = labeled_scale("Contrast", -0.6, 0.6);
    let sat = labeled_scale("Saturation", -0.8, 0.8);
    let exposure = labeled_scale("Exposure", -1.0, 1.0);
    let sharp = labeled_scale("Sharpness", 0.0, 1.0);
    side.append(&heading("Adjust"));
    side.append(&bright.0);
    side.append(&contrast.0);
    side.append(&sat.0);
    side.append(&exposure.0);
    side.append(&sharp.0);
    let apply_adj = gtk::Button::with_label("Apply adjustments");
    let reset_adj = gtk::Button::with_label("Reset sliders");
    reset_adj.add_css_class("flat");
    side.append(&apply_adj);
    side.append(&reset_adj);

    let format = gtk::DropDown::from_strings(&["PNG", "JPEG", "WebP", "AVIF", "TIFF"]);
    format.set_selected(settings.export_format);
    let quality = gtk::Scale::with_range(gtk::Orientation::Horizontal, 40.0, 100.0, 1.0);
    quality.set_value(settings.export_quality as f64);
    quality.set_draw_value(true);
    quality.set_tooltip_text(Some("Quality"));
    let estimate = gtk::Label::new(None);
    estimate.add_css_class("dim-label");
    estimate.set_xalign(0.0);
    let strip = gtk::Switch::new();
    strip.set_active(settings.strip_metadata);
    strip.set_valign(gtk::Align::Center);
    let strip_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let sl = gtk::Label::new(Some("Remove metadata"));
    sl.set_hexpand(true);
    sl.set_xalign(0.0);
    strip_row.append(&sl);
    strip_row.append(&strip);
    let save = gtk::Button::with_label("Save Copy");
    save.add_css_class("suggested-action");
    save.set_tooltip_text(Some("Ctrl+S"));
    let save_as = gtk::Button::with_label("Save Copy As…");
    save_as.set_tooltip_text(Some("Ctrl+Shift+S"));
    side.append(&heading("Export"));
    side.append(&format);
    side.append(&quality);
    side.append(&estimate);
    side.append(&strip_row);
    side.append(&save);
    side.append(&save_as);

    let meta = gtk::Label::new(None);
    meta.set_wrap(true);
    meta.set_xalign(0.0);
    meta.add_css_class("dim-label");
    side.append(&heading("Metadata"));
    side.append(&meta);

    let side_scroll = gtk::ScrolledWindow::new();
    side_scroll.set_child(Some(&side));
    side_scroll.set_min_content_width(320);

    let split = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    split.set_margin_start(12);
    split.set_margin_end(12);
    split.set_margin_bottom(12);
    split.append(&preview);
    split.append(&side_scroll);
    toolbar.set_content(Some(&split));
    toast.set_child(Some(&toolbar));
    window.set_content(Some(&toast));

    let widgets = Rc::new(Widgets {
        window: window.clone(),
        toast,
        picture,
        area,
        dim,
        meta,
        quality,
        format,
        estimate,
        strip,
        bright: bright.1,
        contrast: contrast.1,
        sat: sat.1,
        exposure: exposure.1,
        sharp: sharp.1,
        crop_on,
        w_entry,
        h_entry,
        lock,
    });
    let state = Rc::new(State {
        settings: RefCell::new(settings),
        doc: RefCell::new(None),
        crop: Cell::new(None),
        crop_ratio: Cell::new(0.0),
        drag: Cell::new(None),
    });

    setup_crop_draw(&widgets, &state);
    setup_drop(&widgets, &state);
    let w = widgets.clone();
    widgets.crop_on.connect_toggled(move |btn| {
        w.area.set_visible(btn.is_active());
        w.area.queue_draw();
    });
    widgets.area.set_visible(false);

    let w = widgets.clone();
    let s = state.clone();
    open.connect_clicked(move |_| choose_open(&w, &s));
    let s = state.clone();
    let w = widgets.clone();
    undo.connect_clicked(move |_| {
        if let Some(doc) = s.doc.borrow_mut().as_mut() {
            doc.undo();
        }
        refresh(&w, &s);
    });
    let s = state.clone();
    let w = widgets.clone();
    redo.connect_clicked(move |_| {
        if let Some(doc) = s.doc.borrow_mut().as_mut() {
            doc.redo();
        }
        refresh(&w, &s);
    });
    let s = state.clone();
    let w = widgets.clone();
    rot_l.connect_clicked(move |_| mutate(&w, &s, |d| d.rotate_ccw()));
    let s = state.clone();
    let w = widgets.clone();
    rot_r.connect_clicked(move |_| mutate(&w, &s, |d| d.rotate_cw()));
    let s = state.clone();
    let w = widgets.clone();
    flip_h.connect_clicked(move |_| mutate(&w, &s, |d| d.flip_h()));
    let s = state.clone();
    let w = widgets.clone();
    flip_v.connect_clicked(move |_| mutate(&w, &s, |d| d.flip_v()));
    let s = state.clone();
    let w = widgets.clone();
    apply_crop.connect_clicked(move |_| apply_crop_now(&w, &s));
    let s = state.clone();
    ratio.connect_selected_notify(move |d| {
        s.crop_ratio.set(match d.selected() {
            1 => 1.0,
            2 => 4.0 / 3.0,
            3 => 16.0 / 9.0,
            4 => 3.0 / 2.0,
            _ => 0.0,
        });
    });
    let s = state.clone();
    let w = widgets.clone();
    apply_resize.connect_clicked(move |_| apply_resize_now(&w, &s));
    let s = state.clone();
    let w = widgets.clone();
    rpresets.connect_selected_notify(move |d| apply_preset(&w, &s, d.selected()));
    let s = state.clone();
    let w = widgets.clone();
    apply_adj.connect_clicked(move |_| apply_adjust_now(&w, &s));
    let w = widgets.clone();
    reset_adj.connect_clicked(move |_| reset_sliders(&w));
    let w = widgets.clone();
    let s = state.clone();
    save.connect_clicked(move |_| save_copy(&w, &s, None));
    let w = widgets.clone();
    let s = state.clone();
    save_as.connect_clicked(move |_| save_copy_as(&w, &s));
    let w = widgets.clone();
    let s = state.clone();
    widgets.format.connect_selected_notify(move |_| update_estimate(&w, &s));
    let w = widgets.clone();
    let s = state.clone();
    widgets.quality.connect_value_changed(move |_| update_estimate(&w, &s));
    let w = widgets.clone();
    let s = state.clone();
    let win = widgets.window.clone();
    settings_btn.connect_clicked(move |_| {
        let cur = s.settings.borrow().clone();
        let s = s.clone();
        let w = w.clone();
        dialogs::show_settings(&win, &cur, move |updated| {
            ui::apply_theme(updated.theme);
            s.settings.replace(updated);
            refresh(&w, &s);
        });
    });
    let win = widgets.window.clone();
    about.connect_clicked(move |_| dialogs::show_about(&win));

    add_shortcuts(&window, &widgets, &state);
    if let Some(path) = initial {
        open_path(&widgets, &state, path);
    }
    window.present();
}

fn heading(t: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(t));
    l.add_css_class("heading");
    l.set_xalign(0.0);
    l
}

fn icon_btn(name: &str, tip: &str) -> gtk::Button {
    let b = gtk::Button::from_icon_name(name);
    b.set_tooltip_text(Some(tip));
    b
}

fn labeled_scale(title: &str, min: f64, max: f64) -> (gtk::Box, gtk::Scale) {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let l = gtk::Label::new(Some(title));
    l.set_xalign(0.0);
    let s = gtk::Scale::with_range(gtk::Orientation::Horizontal, min, max, 0.05);
    s.set_value(0.0);
    s.set_draw_value(true);
    box_.append(&l);
    box_.append(&s);
    (box_, s)
}

fn mutate(widgets: &Widgets, state: &State, f: impl FnOnce(&mut Document)) {
    if let Some(doc) = state.doc.borrow_mut().as_mut() {
        f(doc);
    }
    refresh(widgets, state);
}

fn setup_drop(widgets: &Rc<Widgets>, state: &Rc<State>) {
    let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    let w = widgets.clone();
    target.connect_enter(move |_, _, _| {
        w.picture.add_css_class("drop-hover");
        gdk::DragAction::COPY
    });
    let w = widgets.clone();
    target.connect_leave(move |_| w.picture.remove_css_class("drop-hover"));
    let widgets_d = widgets.clone();
    let state_d = state.clone();
    target.connect_drop(move |_, value, _, _| {
        widgets_d.picture.remove_css_class("drop-hover");
        if let Ok(list) = value.get::<gdk::FileList>() {
            if let Some(path) = list.files().iter().find_map(|f| f.path()) {
                open_path(&widgets_d, &state_d, path);
                return true;
            }
        }
        false
    });
    widgets.window.add_controller(target);
}

fn setup_crop_draw(widgets: &Rc<Widgets>, state: &Rc<State>) {
    let state_d = state.clone();
    let picture = widgets.picture.clone();
    widgets.area.set_draw_func(move |_, cr, width, height| {
        if !state_d.doc.borrow().is_some() {
            return;
        }
        let Some(crop) = state_d.crop.get() else {
            return;
        };
        let Some(tex) = picture.paintable() else {
            return;
        };
        let _ = (tex, width, height);
        let (iw, ih) = state_d
            .doc
            .borrow()
            .as_ref()
            .map(|d| d.dimensions())
            .unwrap_or((1, 1));
        let (dx, dy, dw, dh) = fit(iw, ih, width as f64, height as f64);
        let x = dx + crop.x as f64 / iw as f64 * dw;
        let y = dy + crop.y as f64 / ih as f64 * dh;
        let w = crop.w as f64 / iw as f64 * dw;
        let h = crop.h as f64 / ih as f64 * dh;
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        cr.fill().ok();
        cr.set_operator(gtk::cairo::Operator::Clear);
        cr.rectangle(x, y, w, h);
        cr.fill().ok();
        cr.set_operator(gtk::cairo::Operator::Over);
        cr.set_source_rgb(0.37, 0.91, 0.83);
        cr.set_line_width(2.0);
        cr.rectangle(x, y, w, h);
        cr.stroke().ok();
        for (hx, hy) in handles(x, y, w, h) {
            cr.rectangle(hx - 5.0, hy - 5.0, 10.0, 10.0);
            cr.fill().ok();
        }
    });

    let drag = gtk::GestureDrag::new();
    let state_g = state.clone();
    let area = widgets.area.clone();
    drag.connect_drag_begin(move |_, x, y| {
        if let Some(idx) = hit_handle(&state_g, &area, x, y) {
            state_g.drag.set(Some(idx));
        }
    });
    let state_g = state.clone();
    let area = widgets.area.clone();
    drag.connect_drag_update(move |g, dx, dy| {
        let Some(handle) = state_g.drag.get() else {
            return;
        };
        let Some((ox, oy)) = g.start_point() else {
            return;
        };
        update_crop(&state_g, &area, handle, ox + dx, oy + dy);
        area.queue_draw();
    });
    let state_g = state.clone();
    drag.connect_drag_end(move |_, _, _| state_g.drag.set(None));
    widgets.area.add_controller(drag);
}

fn fit(iw: u32, ih: u32, vw: f64, vh: f64) -> (f64, f64, f64, f64) {
    let scale = (vw / iw as f64).min(vh / ih as f64);
    let dw = iw as f64 * scale;
    let dh = ih as f64 * scale;
    ((vw - dw) / 2.0, (vh - dh) / 2.0, dw, dh)
}

fn handles(x: f64, y: f64, w: f64, h: f64) -> [(f64, f64); 8] {
    [
        (x, y),
        (x + w / 2.0, y),
        (x + w, y),
        (x + w, y + h / 2.0),
        (x + w, y + h),
        (x + w / 2.0, y + h),
        (x, y + h),
        (x, y + h / 2.0),
    ]
}

fn hit_handle(state: &State, area: &gtk::DrawingArea, x: f64, y: f64) -> Option<u8> {
    let crop = state.crop.get()?;
    let (iw, ih) = state.doc.borrow().as_ref()?.dimensions();
    let (dx, dy, dw, dh) = fit(iw, ih, area.width() as f64, area.height() as f64);
    let cx = dx + crop.x as f64 / iw as f64 * dw;
    let cy = dy + crop.y as f64 / ih as f64 * dh;
    let cw = crop.w as f64 / iw as f64 * dw;
    let ch = crop.h as f64 / ih as f64 * dh;
    handles(cx, cy, cw, ch)
        .iter()
        .enumerate()
        .find(|(_, (hx, hy))| (x - hx).abs() <= 10.0 && (y - hy).abs() <= 10.0)
        .map(|(i, _)| i as u8)
}

fn update_crop(state: &State, area: &gtk::DrawingArea, handle: u8, x: f64, y: f64) {
    let Some(mut crop) = state.crop.get() else {
        return;
    };
    let Some((iw, ih)) = state.doc.borrow().as_ref().map(|d| d.dimensions()) else {
        return;
    };
    let (dx, dy, dw, dh) = fit(iw, ih, area.width() as f64, area.height() as f64);
    let ix = ((x - dx) / dw * iw as f64).clamp(0.0, iw as f64);
    let iy = ((y - dy) / dh * ih as f64).clamp(0.0, ih as f64);
    match handle {
        0 => {
            let nx = ix.min((crop.x + crop.w - 2) as f64);
            let ny = iy.min((crop.y + crop.h - 2) as f64);
            crop.w = crop.x + crop.w - nx as u32;
            crop.h = crop.y + crop.h - ny as u32;
            crop.x = nx as u32;
            crop.y = ny as u32;
        }
        2 => {
            crop.w = (ix as u32).saturating_sub(crop.x).max(2);
            let ny = iy.min((crop.y + crop.h - 2) as f64);
            crop.h = crop.y + crop.h - ny as u32;
            crop.y = ny as u32;
        }
        4 => {
            crop.w = (ix as u32).saturating_sub(crop.x).max(2);
            crop.h = (iy as u32).saturating_sub(crop.y).max(2);
        }
        6 => {
            let nx = ix.min((crop.x + crop.w - 2) as f64);
            crop.w = crop.x + crop.w - nx as u32;
            crop.x = nx as u32;
            crop.h = (iy as u32).saturating_sub(crop.y).max(2);
        }
        1 => {
            let ny = iy.min((crop.y + crop.h - 2) as f64);
            crop.h = crop.y + crop.h - ny as u32;
            crop.y = ny as u32;
        }
        5 => crop.h = (iy as u32).saturating_sub(crop.y).max(2),
        3 => crop.w = (ix as u32).saturating_sub(crop.x).max(2),
        7 => {
            let nx = ix.min((crop.x + crop.w - 2) as f64);
            crop.w = crop.x + crop.w - nx as u32;
            crop.x = nx as u32;
        }
        _ => {}
    }
    if state.crop_ratio.get() > 0.0 {
        crop.h = ((crop.w as f32 / state.crop_ratio.get()) as u32).max(2).min(ih - crop.y);
    }
    crop.w = crop.w.min(iw - crop.x).max(2);
    crop.h = crop.h.min(ih - crop.y).max(2);
    state.crop.set(Some(crop));
}

fn choose_open(widgets: &Rc<Widgets>, state: &Rc<State>) {
    let dialog = gtk::FileDialog::new();
    dialog.set_title("Open image");
    let filter = gtk::FileFilter::new();
    filter.add_mime_type("image/*");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    let w = widgets.clone();
    let s = state.clone();
    let window = widgets.window.clone();
    dialog.open(Some(&window), gio::Cancellable::NONE, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                open_path(&w, &s, path);
            }
        }
    });
}

fn open_path(widgets: &Widgets, state: &State, path: PathBuf) {
    match Document::open(&path) {
        Ok(doc) => {
            let (w, h) = doc.dimensions();
            state.crop.set(Some(CropRect {
                x: w / 8,
                y: h / 8,
                w: w * 3 / 4,
                h: h * 3 / 4,
            }));
            widgets.w_entry.set_text(&w.to_string());
            widgets.h_entry.set_text(&h.to_string());
            if let Ok(rows) = metadata::summarize(&path) {
                let text = rows
                    .into_iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                widgets.meta.set_text(&text);
            }
            state.doc.replace(Some(doc));
            refresh(widgets, state);
        }
        Err(err) => dialogs::show_error(&widgets.window, &err),
    }
}

fn refresh(widgets: &Widgets, state: &State) {
    let borrowed = state.doc.borrow();
    let Some(doc) = borrowed.as_ref() else {
        return;
    };
    let rgba = doc.rgba8();
    let (w, h) = rgba.dimensions();
    let pixbuf = Pixbuf::from_mut_slice(
        rgba.into_raw(),
        Colorspace::Rgb,
        true,
        8,
        w as i32,
        h as i32,
        w as i32 * 4,
    );
    let texture = gdk::Texture::for_pixbuf(&pixbuf);
    widgets.picture.set_paintable(Some(&texture));
    widgets.dim.set_text(&format!(
        "{}  ·  {w}×{h}{}",
        paths::display_home_path(&doc.source),
        if doc.orientation_applied {
            "  ·  EXIF orientation applied"
        } else {
            ""
        }
    ));
    widgets.area.queue_draw();
    widgets.area.set_visible(widgets.crop_on.is_active());
    update_estimate(widgets, state);
}

fn update_estimate(widgets: &Widgets, state: &State) {
    let borrowed = state.doc.borrow();
    let Some(doc) = borrowed.as_ref() else {
        widgets.estimate.set_text("");
        return;
    };
    let format = ExportFormat::from_index(widgets.format.selected());
    let q = widgets.quality.value() as u8;
    match encode_estimate(&doc.image, format, q) {
        Ok(n) => widgets
            .estimate
            .set_text(&format!("Estimated export about {} KB", (n / 1024).max(1))),
        Err(_) => widgets.estimate.set_text(""),
    }
}

fn apply_crop_now(widgets: &Widgets, state: &State) {
    let Some(rect) = state.crop.get() else {
        return;
    };
    if let Some(doc) = state.doc.borrow_mut().as_mut() {
        if let Err(err) = doc.crop(rect) {
            dialogs::show_error(&widgets.window, &err);
            return;
        }
        let (w, h) = doc.dimensions();
        state.crop.set(Some(CropRect {
            x: w / 10,
            y: h / 10,
            w: w * 8 / 10,
            h: h * 8 / 10,
        }));
        widgets.w_entry.set_text(&w.to_string());
        widgets.h_entry.set_text(&h.to_string());
    }
    refresh(widgets, state);
}

fn apply_resize_now(widgets: &Widgets, state: &State) {
    let (ow, oh) = {
        let borrowed = state.doc.borrow();
        let Some(doc) = borrowed.as_ref() else {
            return;
        };
        doc.dimensions()
    };
    let w = widgets
        .w_entry
        .text()
        .as_str()
        .parse::<u32>()
        .unwrap_or(ow);
    let mut h = widgets
        .h_entry
        .text()
        .as_str()
        .parse::<u32>()
        .unwrap_or(oh);
    if widgets.lock.is_active() && ow > 0 {
        h = ((w as u64 * oh as u64) / ow as u64) as u32;
        widgets.h_entry.set_text(&h.to_string());
    }
    if let Some(doc) = state.doc.borrow_mut().as_mut() {
        if let Err(err) = doc.resize(w, h.max(1)) {
            dialogs::show_error(&widgets.window, &err);
        }
    }
    refresh(widgets, state);
}

fn apply_preset(widgets: &Widgets, state: &State, idx: u32) {
    let (ow, oh) = {
        let borrowed = state.doc.borrow();
        let Some(doc) = borrowed.as_ref() else {
            return;
        };
        doc.dimensions()
    };
    let (w, h) = match idx {
        1 => (1920, ((1920u64 * oh as u64) / ow.max(1) as u64) as u32),
        2 => (1280, ((1280u64 * oh as u64) / ow.max(1) as u64) as u32),
        3 => (800, ((800u64 * oh as u64) / ow.max(1) as u64) as u32),
        4 => (ow / 2, oh / 2),
        _ => return,
    };
    widgets.w_entry.set_text(&w.to_string());
    widgets.h_entry.set_text(&h.max(1).to_string());
}

fn apply_adjust_now(widgets: &Widgets, state: &State) {
    let adj = Adjust {
        brightness: widgets.bright.value() as f32,
        contrast: widgets.contrast.value() as f32,
        saturation: widgets.sat.value() as f32,
        exposure: widgets.exposure.value() as f32,
        sharpness: widgets.sharp.value() as f32,
    };
    if let Some(doc) = state.doc.borrow_mut().as_mut() {
        doc.apply_adjust(adj);
    }
    refresh(widgets, state);
}

fn reset_sliders(widgets: &Widgets) {
    for s in [
        &widgets.bright,
        &widgets.contrast,
        &widgets.sat,
        &widgets.exposure,
        &widgets.sharp,
    ] {
        s.set_value(0.0);
    }
}

fn save_copy(widgets: &Widgets, state: &State, dest_dir: Option<PathBuf>) {
    let borrowed = state.doc.borrow();
    let Some(doc) = borrowed.as_ref() else {
        dialogs::show_error(&widgets.window, &Error::user("Open an image first."));
        return;
    };
    let format = ExportFormat::from_index(widgets.format.selected());
    let q = widgets.quality.value() as u8;
    let strip = widgets.strip.is_active();
    match export::export_copy(
        &doc.image,
        &doc.source,
        format,
        q,
        strip,
        dest_dir.as_deref(),
    ) {
        Ok(path) => {
            let mut st = state.settings.borrow().clone();
            st.export_format = format.index();
            st.export_quality = q;
            st.strip_metadata = strip;
            let _ = st.save();
            state.settings.replace(st);
            widgets.toast.add_toast(adw::Toast::new(&format!(
                "Saved {}",
                paths::display_home_path(&path)
            )));
        }
        Err(err) => dialogs::show_error(&widgets.window, &err),
    }
}

fn save_copy_as(widgets: &Rc<Widgets>, state: &Rc<State>) {
    let dialog = gtk::FileDialog::new();
    dialog.set_title("Save copy into folder");
    let w = widgets.clone();
    let s = state.clone();
    let window = widgets.window.clone();
    dialog.select_folder(Some(&window), gio::Cancellable::NONE, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                save_copy(&w, &s, Some(path));
            }
        }
    });
}

fn add_shortcuts(window: &adw::ApplicationWindow, widgets: &Rc<Widgets>, state: &Rc<State>) {
    let open = gio::SimpleAction::new("open", None);
    let w = widgets.clone();
    let s = state.clone();
    open.connect_activate(move |_, _| choose_open(&w, &s));
    window.add_action(&open);
    let save = gio::SimpleAction::new("save", None);
    let w = widgets.clone();
    let s = state.clone();
    save.connect_activate(move |_, _| save_copy(&w, &s, None));
    window.add_action(&save);
    let save_as = gio::SimpleAction::new("save-as", None);
    let w = widgets.clone();
    let s = state.clone();
    save_as.connect_activate(move |_, _| save_copy_as(&w, &s));
    window.add_action(&save_as);
    let undo = gio::SimpleAction::new("undo", None);
    let w = widgets.clone();
    let s = state.clone();
    undo.connect_activate(move |_, _| {
        if let Some(doc) = s.doc.borrow_mut().as_mut() {
            doc.undo();
        }
        refresh(&w, &s);
    });
    window.add_action(&undo);
    let redo = gio::SimpleAction::new("redo", None);
    let w = widgets.clone();
    let s = state.clone();
    redo.connect_activate(move |_, _| {
        if let Some(doc) = s.doc.borrow_mut().as_mut() {
            doc.redo();
        }
        refresh(&w, &s);
    });
    window.add_action(&redo);
    if let Some(app) = window.application() {
        app.set_accels_for_action("win.open", &["<Ctrl>o"]);
        app.set_accels_for_action("win.save", &["<Ctrl>s"]);
        app.set_accels_for_action("win.save-as", &["<Ctrl><Shift>s"]);
        app.set_accels_for_action("win.undo", &["<Ctrl>z"]);
        app.set_accels_for_action("win.redo", &["<Ctrl><Shift>z"]);
    }
}
