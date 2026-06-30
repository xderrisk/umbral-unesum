mod classrooms;
mod dialogs;
mod firebase;
mod mqtt;
mod news;
mod settings;
mod state;
use adw::prelude::*;
use gettextrs::{bindtextdomain, setlocale, textdomain};
use gtk::{gdk, gio, glib};
use std::cell::Cell;
use std::ops::ControlFlow;
use std::rc::Rc;

fn main() {
    gio::resources_register_include!("resources.gresource")
        .expect("Failed to load embedded resources");

    setlocale(gettextrs::LocaleCategory::LcAll, "");
    let locale_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("locale");
    let locale_path = if locale_dir.exists() {
        locale_dir
    } else {
        std::path::PathBuf::from("locale")
    };
    bindtextdomain(
        "umbral",
        locale_path.to_str().expect("Invalid locale path"),
    )
    .expect("Failed to bind the translation domain");
    textdomain("umbral").expect("Failed to set the text domain");

    let fullscreen = std::rc::Rc::new(Cell::new(false));

    let app = adw::Application::builder()
        .application_id("edu.unesum.umbral")
        .build();

    app.add_main_option(
        "fullscreen",
        glib::Char::from(b'f' as u8),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Start in fullscreen mode",
        None,
    );

    let fullscreen_clone = fullscreen.clone();
    app.connect_handle_local_options(
        move |_app: &adw::Application, dict: &glib::VariantDict| -> ControlFlow<glib::ExitCode> {
            if dict.contains("fullscreen") {
                fullscreen_clone.set(true);
            }
            ControlFlow::Continue(())
        },
    );

    app.connect_activate(move |app| build_ui(app, fullscreen.get()));
    app.run();
}

fn build_ui(app: &adw::Application, fullscreen: bool) {
    let state = state::AppState::new();
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/main_window.ui");

    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("assets/style.css"));
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let window: adw::ApplicationWindow = builder
        .object("window")
        .expect("Main window object not found in UI file");
    window.set_application(Some(app));

    let separator: gtk::Separator = builder.object("main_separator").unwrap();
    let left_container: gtk::Box = builder.object("left_container").unwrap();
    let right_container: gtk::Box = builder.object("right_container").unwrap();

    let news_widget = news::news_section();
    left_container.append(&news_widget);

    let media_unique = gtk::Builder::from_resource("/edu/unesum/umbral/ui/media_section.ui");
    let unique_image_widget: gtk::Picture = media_unique
        .object("media_picture")
        .expect("media_picture not found");
    if !state.borrow().settings.unique_image_path.is_empty() {
        unique_image_widget.set_filename(Some(
            &state.borrow().settings.unique_image_path,
        ));
    }
    left_container.append(&unique_image_widget);

    let (classrooms_section_box, classrooms_relayout) = classrooms::classrooms_section(&state);
    right_container.append(&classrooms_section_box);

    let apply_visibility: Rc<dyn Fn()> = {
        let state = state.clone();
        let news_widget = news_widget.clone();
        let unique_image_widget = unique_image_widget.clone();
        let left_container = left_container.clone();
        let separator = separator.clone();
        Rc::new(move || {
            let s = state.borrow();
            let show_news = s.settings.news;
            let show_image = s.settings.unique_image;
            let path = s.settings.unique_image_path.clone();
            drop(s);
            news_widget.set_visible(show_news);
            unique_image_widget.set_filename(if path.is_empty() {
                None
            } else {
                Some(&path)
            });
            unique_image_widget.set_visible(show_image);
            left_container.set_visible(show_news || show_image);
            separator.set_visible(show_news || show_image);
        })
    };
    apply_visibility();

    let btn_add: gtk::Button = builder.object("btn_add").unwrap();
    btn_add.connect_clicked(glib::clone!(
        #[weak]
        window,
        #[strong]
        state,
        move |_| dialogs::show_add(&window, &state)
    ));

    let btn_full: gtk::Button = builder.object("btn_full").unwrap();
    btn_full.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            if window.is_fullscreen() {
                window.unfullscreen();
            } else {
                window.fullscreen();
            }
        }
    ));

    let btn_config: gtk::Button = builder.object("btn_config").unwrap();
    btn_config.connect_clicked(glib::clone!(
        #[weak]
        window,
        #[strong]
        state,
        #[strong]
        classrooms_relayout,
        #[strong]
        apply_visibility,
        move |_| {
            dialogs::show_config(
                &window,
                &state,
                classrooms_relayout.clone(),
                apply_visibility.clone(),
            );
        }
    ));

    let btn_about: gtk::Button = builder.object("btn_about").unwrap();
    btn_about.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| dialogs::show_about(&window)
    ));

    window.present();

    if fullscreen {
        window.fullscreen();
    }
}
