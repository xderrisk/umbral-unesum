use adw::prelude::*;

pub fn show_about(parent: &adw::ApplicationWindow) {
    let builder = gtk::Builder::from_resource("/edu/unesum/umbral/ui/about_dialog.ui");
    let dialog: adw::AboutDialog = builder
        .object("about_dialog")
        .expect("No about_dialog found");
    dialog.present(Some(parent));
}
