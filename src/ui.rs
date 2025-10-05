use super::uupd;

use futures::FutureExt;
use gtk::prelude::*;
use relm4::*;

use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};

#[derive(Default)]
pub struct App {
    /// Tracks if we're updating
    updating: bool,

    /// Track if we're rebooting after an OS upgrade
    reboot: bool,

    /// Contains output of a completed task.
    task: Option<CmdOut>,
}

pub struct Widgets {
    update: gtk::Button,
    apply: gtk::CheckButton,
    overall_progress: gtk::ProgressBar,
}

#[derive(Debug)]
pub enum Input {
    Update,
    Apply,
}

#[derive(Debug)]
pub enum Output {}

// {"msg":"Updating","title":"System","description":"rpm-ostree","progress":0,"total":5,"step_progress":0,"overall":17}
#[derive(Debug, Clone)]
pub enum CmdOut {
    /// Progress update from a command.
    Json(uupd::Progress, u32),
    /// The final output of the command.
    Finished,
}

impl Component for App {
    type Init = String;
    type Input = Input;
    type Output = Output;
    type CommandOutput = CmdOut;
    type Widgets = Widgets;
    type Root = gtk::Window;

    fn init_root() -> Self::Root {
        gtk::Window::default()
    }

    fn init(
        _args: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        relm4::view! {
            container = gtk::Box {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_width_request: 300,
                set_spacing: 12,
                set_margin_top: 4,
                set_margin_bottom: 4,
                set_margin_start: 12,
                set_margin_end: 12,
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    set_spacing: 4,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,

                    append: update = &gtk::Button {
                        set_label: "Update",
                        connect_clicked => Input::Update,
                    },

                    append: apply = &gtk::CheckButton {
                        set_label: Some("Auto-reboot if the OS image is updated?"),
                        connect_toggled => Input::Apply,
                    },

                    append: overall_progress = &gtk::ProgressBar {
                        set_visible: false,
                        set_show_text: true,

                    }
                },
            }
        }

        root.set_child(Some(&container));

        ComponentParts {
            model: App::default(),
            widgets: Widgets {
                update,
                apply,
                overall_progress,
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Apply => {
                if !self.updating {
                    self.reboot = !self.reboot;
                }
            }
            Input::Update => {
                self.updating = true;

                sender.command(|out, shutdown| {
                    shutdown
                        // Performs this operation until a shutdown is triggered
                        .register(async move {
                            let cmd = "pkexec uupd --json".to_string();
                            if let Ok(child_process) = Command::new("sh")
                                .args(["-c", &cmd])
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                let incoming = child_process.stdout.unwrap();
                                let mut previous_overall = 0;

                                let _ = &BufReader::new(incoming).lines().for_each(|line| {
                                    let data = line.unwrap();
                                    // println!("{}", data);
                                    let p: uupd::Progress = serde_json::from_str(&data).unwrap();

                                    // There'll be no progress or total on the last message
                                    let finished = p.progress == 0 && p.total == 0;
                                    if !finished {
                                        out.send(CmdOut::Json(p.clone(), previous_overall))
                                            .unwrap();
                                        previous_overall = p.overall;
                                    } else {
                                        out.send(CmdOut::Finished).unwrap();
                                    }
                                });
                            }
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let CmdOut::Finished = message {
            if self.reboot {
                // I'm not sure if this is helping or not, tbh, but I can only test once/day.
                std::thread::sleep(std::time::Duration::from_secs(3));

                let cmd = "systemctl reboot".to_string();
                Command::new("sh")
                    .args(["-c", &cmd])
                    .output()
                    .expect("Failed to reboot");
            } else {
                self.updating = false;
            }
        }
        self.task = Some(message);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.update.set_sensitive(!self.updating);
        widgets.apply.set_sensitive(!self.updating);

        if let Some(ref progress) = self.task {
            match progress {
                CmdOut::Json(received, previous_overall) => {
                    // println!("{:?}", received);

                    let progress = if (received.progress + 1) < received.total {
                        received.progress + 1
                    } else {
                        received.progress
                    };

                    let msg = format!(
                        "{} {} - {} (step {}/{})...",
                        received.msg,
                        received.title,
                        received.description,
                        progress,
                        received.total + 1
                    );

                    widgets.overall_progress.set_text(Some(&msg));
                    widgets.overall_progress.set_visible(true);
                    widgets
                        .overall_progress
                        .set_fraction(*previous_overall as f64 / 100.0);
                }
                CmdOut::Finished => {
                    let msg = format!(
                        "Update complete! {}",
                        if self.reboot { "Rebooting..." } else { "" }
                    );
                    widgets.overall_progress.set_text(Some(&msg));
                    widgets.overall_progress.set_fraction(1.0);
                }
            }
        }
    }
}
