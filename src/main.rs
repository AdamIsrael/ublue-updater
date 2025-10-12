mod actions;
mod config;
mod modals;
mod ui;
mod utils;
mod uupd;

use uupd::Progress;

use gtk::prelude::*;

use std::cell::RefCell;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
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
    let progress_bar = ui::get_progress_bar();
    let apply_check_button = ui::get_apply_check_button();

    // Create cloned references because the closure will move them
    let pbar = progress_bar.clone();
    let apply = apply_check_button.clone();
    let update = update_button.clone();

    // Connect update button to run a system update command
    update_button.connect_clicked(move |_| {
        // Get the UI elements that the secondary thread needs to access
        let ui_model = ui::UiModel {
            apply_check_button: apply.clone(),
            update_button: update.clone(),
            progress_bar: pbar.clone(),
        };

        execute_command_async(ui_model);
    });

    let main_box = ui::get_main_container(
        &header_bar,
        &update_button,
        &apply_check_button,
        &progress_bar,
    );
    let window = ui::get_window(app, "UBlue Updater", main_box);

    // Now that we have the window, connect the menu actions
    actions::set_about(&app, &window);
    actions::set_quit(&app);

    // Present window
    window.present();
}

fn execute_command_async(ui: ui::UiModel) {
    // Disable the update button and checkbox while running uupd
    ui.apply_check_button.set_sensitive(false);
    ui.update_button.set_sensitive(false);
    ui.progress_bar.set_visible(true);

    let (tx, rx) = mpsc::channel();
    GLOBAL.with(|global| {
        *global.borrow_mut() = Some((ui, rx));
    });

    let cmd = "pkexec uupd --json".to_string();
    if let Ok(child_process) = Command::new("sh")
        .args(["-c", &cmd])
        .stdout(Stdio::piped())
        .spawn()
    {
        let incoming = child_process.stdout.unwrap();

        let mut previous_overall = 0;

        std::thread::spawn(move || {
            let _ = &BufReader::new(incoming).lines().for_each(|line| {
                let data = line.unwrap();
                let mut p: Progress = serde_json::from_str(&data).unwrap();

                p.previous_overall = previous_overall;

                // Track the previous progress
                previous_overall = p.overall;

                // send data through channel
                tx.send(p).unwrap();

                // then tell the UI thread to read from that channel
                glib::source::idle_add(|| {
                    check_for_new_message();
                    glib::ControlFlow::Break
                });
            });
        });
    }
}

// global variable to store the ui and an input channel
// on the main thread only
thread_local!(
    static GLOBAL: RefCell<Option<(ui::UiModel, mpsc::Receiver<uupd::Progress>)>> =
        const { RefCell::new(None) };
);

// function to check if a new message has been passed through the
// global receiver and, if so, add it to the UI.
fn check_for_new_message() {
    GLOBAL.with(|global| {
        if let Some((ui, rx)) = &*global.borrow() {
            // Receive the Progress struct
            let p: uupd::Progress = rx.recv().unwrap();

            // Update the UI
            let progress = if (p.progress + 1) < p.total {
                p.progress + 1
            } else {
                p.progress
            };
            println!("Progress: {}/{}", progress, p.total);

            let msg = format!(
                "{} {} - {} (step {}/{})...",
                p.msg,
                p.title,
                p.description,
                progress,
                p.total + 1
            );
            ui.progress_bar.set_text(Some(&msg));
            ui.progress_bar
                .set_fraction(p.previous_overall as f64 / 100.0);

            let finished = p.progress == 0 && p.total == 0;

            // If the progress is complete, re-enable the disabled UI elements
            if finished {
                ui.update_button.set_sensitive(true);
                ui.apply_check_button.set_sensitive(true);
                // ui.progress_bar.set_visible(false);

                let reboot = ui.apply_check_button.is_active();

                let msg = format!(
                    "Update complete! {}",
                    if reboot { "Rebooting..." } else { "" }
                );
                ui.progress_bar.set_text(Some(&msg));
                ui.progress_bar.set_fraction(1.0);

                if reboot && utils::check_reboot_needed() {
                    utils::reboot_system();
                }
            }
        }
    });
}
