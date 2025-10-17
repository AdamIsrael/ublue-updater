mod actions;
mod config;
mod modals;
mod ui;
mod utils;
mod uupd;

use renovatio::Plugin;

use gtk::prelude::*;
use libloading::{Library, Symbol};

use std::cell::RefCell;
use std::sync::mpsc;

fn main() -> glib::ExitCode {
    // Initialize our GSettings schema, if it doesn't exist
    utils::install_gsettings_schema();

    // Create a new application
    let application = adw::Application::builder()
        .application_id(config::APP_ID)
        .build();

    application.connect_activate(|app| {
        build_ui(app);
    });

    // Run the application
    application.run()
}

fn build_ui(app: &adw::Application) {
    let header_bar = ui::get_header_bar();
    let update_button = ui::get_update_button();
    let plugin_progress_bar = ui::get_plugin_progress_bar();
    let total_progress_bar = ui::get_total_progress_bar();
    let apply_check_button = ui::get_apply_check_button();

    // Create cloned references because the closure will move them
    let tpbar = total_progress_bar.clone();
    let ppbar = plugin_progress_bar.clone();
    let apply = apply_check_button.clone();
    let update = update_button.clone();

    // Connect update button to run a system update command
    update_button.connect_clicked(move |_| {
        // Get the UI elements that the secondary thread needs to access
        let ui_model = ui::UiModel {
            plugin_count: 0,
            apply_check_button: apply.clone(),
            update_button: update.clone(),
            plugin_progress_bar: ppbar.clone(),
            total_progress_bar: tpbar.clone(),
        };

        execute_command_async(ui_model);
    });

    let main_box = ui::get_main_container(
        &header_bar,
        &update_button,
        &apply_check_button,
        &plugin_progress_bar,
        &total_progress_bar,
    );
    let window = ui::get_window(app, "Renovatio", main_box);

    // Now that we have the window, connect the menu actions
    actions::set_about(&app, &window);
    actions::set_quit(&app);

    // Present window
    window.present();
}

fn execute_command_async(mut ui: ui::UiModel) {
    // Disable the update button and checkbox while running uupd
    ui.apply_check_button.set_sensitive(false);
    ui.update_button.set_sensitive(false);
    ui.plugin_progress_bar.set_visible(true);
    ui.total_progress_bar.set_visible(true);
    // ui.plugin_progress_bar.set_pulse_step(0.05);
    // ui.plugin_progress_bar.pulse();

    // Load uupd plugin
    let mut plugins = utils::find_plugins();
    ui.plugin_count = plugins.len() as u32;

    // Create the channel for communication between the main thread and the plugin thread
    let (tx, rx) = mpsc::channel();
    GLOBAL.with(|global| {
        *global.borrow_mut() = Some((ui, rx));
    });

    // TODO: Iterate through all of the _enabled_ plugins
    let plugin = plugins.pop().unwrap();

    // Execute each plugin in a separate thread so we don't block the main thread
    std::thread::spawn(move || {
        unsafe {
            if let Ok(lib) = Library::new(plugin) {
                let result: Result<Symbol<unsafe fn() -> *mut dyn Plugin>, _> =
                    lib.get(b"create_plugin\0");
                if let Ok(create_plugin) = result {
                    let plugin_ptr = create_plugin();
                    let plugin: Box<dyn Plugin> = Box::from_raw(plugin_ptr); // Reclaim ownership

                    println!("Plugin Name: {}", plugin.name());

                    if plugin.update(tx, tick) {
                        println!("Update successful");
                    } else {
                        println!("Update failed");
                    }
                }
            }
        };
    });
}

extern "Rust" fn tick() {
    // Tell the UI thread to read from the channel
    glib::source::idle_add(|| {
        check_for_new_message();
        glib::ControlFlow::Break
    });
}

// global variable to store the ui and an input channel
// on the main thread only
thread_local!(
    static GLOBAL: RefCell<Option<(ui::UiModel, mpsc::Receiver<renovatio::Progress>)>> =
        const { RefCell::new(None) };
);

// function to check if a new message has been passed through the
// global receiver and, if so, add it to the UI.
fn check_for_new_message() {
    GLOBAL.with(|global| {
        if let Some((ui, rx)) = &*global.borrow() {
            // Receive the Progress struct
            let res = rx.recv();
            if let Err(e) = res {
                println!("Error receiving progress: {}", e);
                return;
            }

            // let p: uupd::Progress = rx.recv().unwrap();
            let p: renovatio::Progress = res.unwrap();
            println!("progress: {:?}", p);

            // Plugin progress goes to the plugin_progress_bar
            ui.plugin_progress_bar.set_text(Some(&p.status));
            ui.plugin_progress_bar
                .set_fraction(p.progress as f64 / 100.0);

            // TODO: Calculate the total progress based on plugin(s) completed
            ui.total_progress_bar.set_text(Some(&p.status));
            ui.total_progress_bar
                .set_fraction(p.progress as f64 / 100.0);

            let finished = p.progress == 100;

            // If the progress is complete, re-enable the disabled UI elements
            if finished {
                ui.update_button.set_sensitive(true);
                ui.apply_check_button.set_sensitive(true);

                let reboot = ui.apply_check_button.is_active();

                let msg = format!(
                    "Updates complete! {}",
                    if reboot { "Rebooting..." } else { "" }
                );

                // ui.plugin_progress_bar.set_pulse_step(0.0);

                ui.total_progress_bar.set_text(Some(&msg));
                ui.total_progress_bar.set_fraction(1.0);

                if reboot && utils::check_reboot_needed() {
                    utils::reboot_system();
                }
            }
        }
    });
}
