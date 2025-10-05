mod ui;
mod utils;
mod uupd;

use relm4::*;

fn main() {
    RelmApp::new("com.github.AdamIsrael.UblueUpdater").run::<ui::App>("Ublue Updater".into());
}
