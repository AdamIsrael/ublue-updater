use crate::config;

use adw::prelude::{AdwDialogExt, IsA};

pub fn show(parent: &impl IsA<gtk::Widget>) {
    let dialog = adw::AboutDialog::builder()
        .application_icon(config::APP_ID)
        .license_type(gtk::License::MitX11)
        .website("https://github.com/AdamIsrael/ublue-updater/")
        .issue_url("https://github.com/AdamIsrael/ublue-updater/issues")
        .application_name("Ublue Updater")
        .version(config::VERSION)
        .build();

    dialog.present(Some(parent));
}
