use crate::config;

use adw::prelude::{AdwDialogExt, IsA};

pub fn show(parent: &impl IsA<gtk::Widget>) {
    let dialog = adw::AboutDialog::builder()
        .application_icon(config::APP_ID)
        .license_type(gtk::License::MitX11)
        .website("https://github.com/AdamIsrael/renovatio/")
        .issue_url("https://github.com/AdamIsrael/renovatio/issues")
        .application_name("Renovatio")
        .version(config::VERSION)
        .build();

    dialog.present(Some(parent));
}
