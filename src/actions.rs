use crate::modals;
use adw::prelude::*;
use gio::ActionEntry;

/// Set the about action
pub fn set_about(app: &adw::Application, window: &adw::ApplicationWindow) {
    app.add_action_entries([ActionEntry::builder("about")
        .activate(glib::clone!(
            #[weak]
            window,
            move |_app: &adw::Application, _action, _parameter| {
                modals::about::show(&window);
            }
        ))
        .build()]);
}

/// Set the quit action
pub fn set_quit(app: &adw::Application) {
    app.add_action_entries([ActionEntry::builder("quit")
        .activate(move |app: &adw::Application, _action, _parameter| {
            app.quit();
        })
        .build()]);
}
