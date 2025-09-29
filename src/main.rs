use duct::cmd;
use gtk::CssProvider;
use gtk::{Application, ApplicationWindow, Button, TextBuffer, TextView, glib};
use gtk::{Expander, prelude::*};

use std::io::BufRead;
use std::io::BufReader;
// use std::io::Read;
// use std::io::prelude::*;
// use std::process::Command;
// use std::process::Stdio;
use std::str;
// use vte4::prelude::*;

const APP_ID: &str = "com.github.AdamIsrael.Updater";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create buttons for different commands
    let update_button = Button::builder()
        .label("Run System Update")
        .margin_top(12)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Create PTY for the terminal
    // gio::Initable::new();
    // let pty = vte4::Pty::builder().build();
    // gio::Initable::new(Some(&pty)).unwrap();
    //
    // Create terminal widget
    // let terminal = vte4::Terminal::builder()
    //     .input_enabled(true)
    //     .scroll_on_output(true)
    //     // .pty(&pty)
    //     .build();

    // // Clone terminal for button callback
    // let terminal_update = terminal.clone();

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
    // terminal.style_context().add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let buffer = terminal.buffer();

    // terminal.set_editable(false);
    // Connect update button to run a system update command
    update_button.connect_clicked(move |_| {
        execute_command_async(
            &buffer,
            "ujust update",
            // "echo 'Starting system update...' && brew update && brew upgrade && echo 'Update completed!'",
        );
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
}

fn execute_command_async(buffer: &TextBuffer, command: &str) {
    // fn execute_command_async(terminal: &vte4::Terminal, command: &str) {
    // let terminal_clone = terminal.clone();
    let command_string = command.to_string();
    let tbuffer = buffer.clone();

    // Execute command in a separate thread to avoid blocking the UI
    glib::spawn_future_local(async move {
        // Show the command being executed
        let cmd_display = format!("$ {}\n", command_string);
        tbuffer.insert_at_cursor(&cmd_display);
        tbuffer.insert_at_cursor("\n");

        // terminal_clone.feed(cmd_display.as_bytes());
        // terminal_clone.feed(b"\n");

        // duct does the heavy lifting of executing the command and handling its output
        // there's probably a way to do it with the standard process::Command but this was faster
        let big_cmd = cmd!("bash", "-c", command_string); // , "1>&2"
        if let Ok(reader) = big_cmd.stderr_to_stdout().reader() {
            let lines = BufReader::new(reader).lines();
            for line in lines {
                if let Ok(line) = line {
                    println!("Got line: {}", line); // debug
                    tbuffer.insert_at_cursor(&line);
                    tbuffer.insert_at_cursor("\n");

                    // terminal_clone.feed(line.as_bytes());

                    // terminal_clone.feed(b"\n");
                }
            }
        }
    });
}
