use duct::cmd;
use gtk::CssProvider;
use gtk::{Application, ApplicationWindow, Button, ProgressBar, TextBuffer, TextView, glib};
use gtk::{Expander, prelude::*};

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
#[derive(Serialize, Deserialize)]
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

struct UiModel {
    progress_bar: ProgressBar,
    output_buffer: TextBuffer,
}

fn main() -> glib::ExitCode {
    // Create a new application
    let application = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    // app.connect_activate(build_ui);

    application.connect_activate(|app| {
        let ui_model = build_ui(app);
        // setup(ui_model);
    });

    // Run the application
    application.run()
}

fn build_ui(app: &Application) -> UiModel {
    let update_button = Button::builder()
        .label("Run System Update")
        .margin_top(12)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Create a progress bar
    let progress_bar = ProgressBar::builder()
        .margin_top(12)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .show_text(true)
        .build();

    // Create a terminal view
    let terminal = TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .monospace(true)
        .build();

    let css_data = "
            textview {
                background-color: black;
                color: white;
            }
            textview text {
                font-family: 'Monospace';
                font-size: 14px;
            }
        ";

    terminal.add_css_class("textview");

    let css_provider = CssProvider::new();
    css_provider.load_from_string(css_data);

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not get display"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // let buffer = terminal.buffer();
    let pbar = progress_bar.clone();
    let tm = terminal.clone();

    // Connect update button to run a system update command
    update_button.connect_clicked(move |_| {
        let ui_model = UiModel {
            progress_bar: pbar.clone(),
            output_buffer: tm.buffer(),
        };
        // execute_command_async(&pbar, &buffer, "uupd --json");
        setup(ui_model);
    });

    // Create expander for terminal
    let expander = Expander::builder()
        .label("Terminal Output")
        .expanded(true) // set to true for debugging
        .build();
    expander.set_child(Some(&terminal));

    // Create main container
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
    main_box.append(&update_button);
    main_box.append(&progress_bar);
    main_box.append(&expander);

    // Create window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("UBlue Updater")
        .default_width(800)
        .default_height(600)
        .child(&main_box)
        .build();

    // Present window
    window.present();

    return UiModel {
        progress_bar: progress_bar.clone(),
        output_buffer: terminal.buffer(),
    };
}

fn setup(ui: UiModel) {
    let (tx, rx) = mpsc::channel();
    GLOBAL.with(|global| {
        *global.borrow_mut() = Some((ui, rx));
    });
    let child_process = Command::new("sh")
        // .args(&["-c", "while true; do date; sleep 2; done"])
        .args(&["-c", "uupd --json"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let incoming = child_process.stdout.unwrap();

    std::thread::spawn(move || {
        let _ = &BufReader::new(incoming).lines().for_each(|line| {
            let data = line.unwrap();
            // send data through channel
            tx.send(data).unwrap();
            // then tell the UI thread to read from that channel
            glib::source::idle_add(|| {
                check_for_new_message();
                return glib::ControlFlow::Break;
            });
        });
    });
}

// global variable to store  the ui and an input channel
// on the main thread only
thread_local!(
    static GLOBAL: RefCell<Option<(UiModel, mpsc::Receiver<String>)>> = RefCell::new(None);
);

// function to check if a new message has been passed through the
// global receiver and, if so, add it to the UI.
fn check_for_new_message() {
    GLOBAL.with(|global| {
        if let Some((ui, rx)) = &*global.borrow() {
            let received: String = rx.recv().unwrap();
            println!("Received message: {}", received);
            let p: Progress = serde_json::from_str(&received).unwrap();
            // Update the progress bar
            ui.progress_bar
                .set_text(Some(&format!("{} {} ({})", p.msg, p.title, p.description)));
            // need to figure out if/when to use fraction vs. pulse_step
            ui.progress_bar.set_fraction(p.overall as f64 / 100.0);

            // ui.output_buffer.set_text(&received);
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
