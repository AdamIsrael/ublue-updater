mod config;
mod ui;
mod utils;
mod uupd;

use relm4::*;

fn main() {
    // Initialize our GSettings schema, if it doesn't exist
    utils::install_gsettings_schema();

    // Run the application
    RelmApp::new("com.github.AdamIsrael.UblueUpdater").run::<ui::App>("Ublue Updater".into());
}
