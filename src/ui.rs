/// UI-related functions and structs
use gtk::CssProvider;
use gtk::{Application, ApplicationWindow, Box, Button, ProgressBar, TextBuffer, TextView};
use gtk::{Expander, prelude::*};

// This provides us a way to update the UI from another thread
pub struct UiModel {
    pub progress_bar: ProgressBar,
    pub output_buffer: TextBuffer,
}

pub fn get_expander(terminal: &TextView) -> Expander {
    let expander = Expander::builder()
        .label("Terminal Output")
        .expanded(true) // set to true for debugging
        .build();
    expander.set_child(Some(terminal));
    expander
}
pub fn get_progress_bar() -> ProgressBar {
    ProgressBar::builder()
        .margin_top(12)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
        .show_text(true)
        .build()
}

pub fn get_terminal_view() -> TextView {
    let terminal = TextView::builder()
        .margin_top(12)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(12)
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
    terminal
}

pub fn get_update_button() -> Button {
    Button::builder()
        .label("Run System Update")
        .margin_top(12)
        .margin_bottom(6)
        .build()
}

pub fn get_main_container(
    update_button: &Button,
    progress_bar: &ProgressBar,
    expander: &Expander,
) -> Box {
    // Create main container
    let main_box = Box::new(gtk::Orientation::Vertical, 6);
    main_box.append(update_button);
    main_box.append(progress_bar);
    main_box.append(expander);

    main_box
}
pub fn get_window(app: &Application, title: &str, main_box: &Box) -> ApplicationWindow {
    // Create window
    ApplicationWindow::builder()
        .application(app)
        .title(title)
        .default_width(800)
        .default_height(600)
        .child(main_box)
        .build()
}
