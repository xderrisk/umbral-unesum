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

fn main() {
    gio::resources_register_include!("resources.gresource")
        .expect("Failed to load embedded resources");

    setlocale(gettextrs::LocaleCategory::LcAll, "");
    let mut locale_dir = std::env::current_exe().unwrap();
    locale_dir.set_file_name("locale");
    bindtextdomain(
        "umbral",
        locale_dir.to_str().expect("Invalid locale path"),
    )
    .expect("Failed to bind the translation domain");
    textdomain("umbral").expect("Failed to set the text domain");

    let app = adw::Application::builder()
        .application_id("edu.unesum.umbral")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
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

    left_container.append(&news::news_section());
    let (classrooms_section_box, classrooms_relayout) = classrooms::classrooms_section(&state);
    right_container.append(&classrooms_section_box);

    let is_news_enabled = state.borrow().settings.news;
    left_container.set_visible(is_news_enabled);
    separator.set_visible(is_news_enabled);

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
        #[weak]
        left_container,
        #[weak]
        separator,
        #[strong]
        classrooms_relayout,
        move |_| {
            dialogs::show_config(
                &window,
                &state,
                left_container.clone(),
                separator.clone(),
                classrooms_relayout.clone(),
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
}
