mod ui;

use gtk::Application;
use gtk::prelude::*;

use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::str;
use std::sync::mpsc;

const APP_ID: &str = "com.github.AdamIsrael.Updater";

// {"level":"INFO","msg":"Updating","title":"System","description":"rpm-ostree",
// "progress":0,"total":5,"step_progress":0,"overall":17}
#[derive(Deserialize, Serialize)]
struct Progress {
    level: String,
    msg: String,

    #[serde(default)]
    title: String,

    #[serde(default)]
    description: String,

    #[serde(default)]
    progress: u32,

    #[serde(default)]
    total: u32,

    #[serde(default)]
    step_progress: u32,

    #[serde(default)]
    overall: u32,
}

fn main() -> glib::ExitCode {
    // Create a new application
    let application = Application::builder().application_id(APP_ID).build();

    application.connect_activate(|app| {
        build_ui(app);
    });

    // Run the application
    application.run()
}

fn build_ui(app: &Application) {
    let update_button = ui::get_update_button();
    let progress_bar = ui::get_progress_bar();
    let terminal = ui::get_terminal_view();
    let expander = ui::get_expander(&terminal);

    // Create cloned references because the closure will move them
    let pbar = progress_bar.clone();
    let term = terminal.clone();

    // Connect update button to run a system update command
    update_button.connect_clicked(move |_| {
        // Get the UI elements that the secondary thread needs to access
        let ui_model = ui::UiModel {
            progress_bar: pbar.clone(),
            output_buffer: term.buffer(),
        };
        execute_command_async(ui_model);
    });

    let main_box = ui::get_main_container(&update_button, &progress_bar, &expander);
    let window = ui::get_window(app, "UBlue Updater", &main_box);

    // Present window
    window.present();
}

fn execute_command_async(ui: ui::UiModel) {
    let (tx, rx) = mpsc::channel();
    GLOBAL.with(|global| {
        *global.borrow_mut() = Some((ui, rx));
    });

    // TODO: Read from UI checkbox to see if we should add the `--apply` flag
    //
    if let Ok(child_process) = Command::new("sh")
        .args(["-c", "uupd --json"])
        .stdout(Stdio::piped())
        .spawn()
    {
        let incoming = child_process.stdout.unwrap();

        std::thread::spawn(move || {
            let _ = &BufReader::new(incoming).lines().for_each(|line| {
                let data = line.unwrap();
                // send data through channel
                tx.send(data).unwrap();
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
    static GLOBAL: RefCell<Option<(ui::UiModel, mpsc::Receiver<String>)>> =
        const { RefCell::new(None) };
);

// function to check if a new message has been passed through the
// global receiver and, if so, add it to the UI.
fn check_for_new_message() {
    GLOBAL.with(|global| {
        if let Some((ui, rx)) = &*global.borrow() {
            let received: String = rx.recv().unwrap();
            // println!("Received message: {}", received);

            // Parse the received json into a Progress struct
            let p: Progress = serde_json::from_str(&received).unwrap();

            // Update the UI
            ui.progress_bar
                .set_text(Some(&format!("{} {} ({})", p.msg, p.title, p.description)));
            ui.progress_bar.set_fraction(p.overall as f64 / 100.0);

            ui.output_buffer.insert_at_cursor(&received);
            ui.output_buffer.insert_at_cursor("\n");
        }
    });
}

// fn execute_command_async(progress_bar: &ProgressBar, buffer: &TextBuffer, command: &str) {
//     let command_string = command.to_string();
//     let tbuffer = buffer.clone();
//     let pbar = progress_bar.clone();

//     // Execute command in a separate thread to avoid blocking the UI
//     glib::spawn_future_local(async move {
//         // Show the command being executed
//         let cmd_display = format!("$ {}\n", command_string);
//         tbuffer.insert_at_cursor(&cmd_display);
//         tbuffer.insert_at_cursor("\n");

//         // duct does the heavy lifting of executing the command and handling its output
//         // there's probably a way to do it with the standard process::Command but this was faster
//         let big_cmd = cmd!("bash", "-c", command_string); // , "1>&2"
//         if let Ok(reader) = big_cmd.stderr_to_stdout().reader() {
//             let lines = BufReader::new(reader).lines();
//             for line in lines {
//                 if let Ok(line) = line {
//                     // parse json
//                     // TODO: handle parsing errors
//                     let p: Progress = serde_json::from_str(&line).unwrap();

//                     // Update the progress bar
//                     pbar.set_text(Some(&format!("{} {} ({})", p.msg, p.title, p.description)));
//                     // need to figure out if/when to use fraction vs. pulse_step
//                     pbar.set_fraction(p.overall as f64 / 100.0);

//                     println!("Got line: {}", line); // debug
//                     tbuffer.insert_at_cursor(&line);
//                     tbuffer.insert_at_cursor("\n");
//                 }
//             }
//         }
//     });
// }
