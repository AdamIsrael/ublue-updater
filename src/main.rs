mod actions;
mod config;
mod modals;
mod ui;
mod utils;

use flume::{Receiver, unbounded};
use renovatio::{Plugin, PluginMetadata, PluginProgress};

use gtk::prelude::*;
use libloading::{Library, Symbol};

// use std::cell::RefCell;
// use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type PluginType = unsafe fn() -> *mut dyn Plugin;

fn main() -> glib::ExitCode {
    // Initialize our GSettings schema, if it doesn't exist
    utils::install_gsettings_schema();

    // Create a new application
    let application = adw::Application::builder()
        .application_id(config::APP_ID)
        .build();

    // Load plugins at startup: we'll need them for the update process and preferences dialog
    // trying to store the plugin itself in a vector, stuffed in a Vec<Box<dyn Plugin>>,
    // into application.data, but I segfault trying to access the plugin data.
    let mut plugins: Vec<PluginMetadata> = Vec::new();
    unsafe {
        let plugin_libraries = utils::find_plugins();
        for plugin_path in &plugin_libraries {
            if let Ok(lib) = Library::new(plugin_path) {
                let result: Result<Symbol<PluginType>, _> = lib.get(b"create_plugin\0");
                if let Ok(create_plugin) = result {
                    let plugin_ptr = create_plugin();
                    let plugin: Box<dyn Plugin> = Box::from_raw(plugin_ptr); // Reclaim ownership
                    let mut metadata = PluginMetadata::new(plugin);
                    metadata.path = Some(plugin_path.clone());
                    plugins.push(metadata);
                }
            }
        }

        // app.set_data::<u32>("counter", Box::new(42));
        // println!("{:?}", plugins);
        // application.set_data::<Vec<PluginMetadata>>("plugins", plugins);
        // application.data::<Vec<PluginMetadata>>("plugins")
    }

    application.connect_activate(move |app| {
        let window = build_ui(app, plugins.clone());

        // Connect to the "close-request" signal
        window.connect_close_request(move |window| {
            // If the window closes, quit the application even if there are active sender channels
            window.application().unwrap().quit();

            // Return `glib::Propagation::Stop` to prevent the default handler
            // from closing the window, or `glib::Propagation::Proceed` to allow it.
            // For example, you might show a confirmation dialog here.
            glib::Propagation::Proceed
        });
    });

    // Run the application
    application.run()
}

fn build_ui(app: &adw::Application, plugins: Vec<PluginMetadata>) -> adw::ApplicationWindow {
    // Create a channel that will be used to send messages from worker threads
    let (tx, rx): (flume::Sender<PluginProgress>, Receiver<PluginProgress>) = unbounded();

    let header_bar = ui::get_header_bar();
    let update_button = ui::get_update_button();
    let plugin_progress_bar = ui::get_plugin_progress_bar();
    let total_progress_bar = ui::get_total_progress_bar();
    let apply_check_button = ui::get_apply_check_button();

    // Create cloned references because the closure will capture them
    let tpbar = total_progress_bar.clone();
    let ppbar = plugin_progress_bar.clone();
    let apply = apply_check_button.clone();
    let update = update_button.clone();

    // Clone handles for the closure that will be run in a new thread
    let tx_clone = tx.clone();

    // Connect update button to run a system update command
    update_button.connect_clicked(move |_| {
        // Disable the update button and checkbox while running updates
        apply.set_sensitive(false);
        update.set_sensitive(false);
        ppbar.set_visible(true);
        tpbar.set_visible(true);

        let tx_worker = tx_clone.clone();
        // drop(tx_worker);

        // let update_clone = update.clone();
        thread::spawn(move || {
            // std::thread::sleep(std::time::Duration::from_millis(5000));

            let settings = gio::Settings::new(config::APP_ID);

            // Load the enabled plugin(s)
            let plugins = settings.get::<Vec<String>>("plugins");

            for plugin in plugins {
                let tx_plugin = tx_worker.clone();
                // let tx_plugin = tx_worker.clone();
                // let (tx_plugin, _rx): (flume::Sender<Progress>, Receiver<Progress>) = unbounded();

                unsafe {
                    // Load the shared library
                    if let Ok(lib) = Library::new(plugin) {
                        // Instantiate the plugin
                        let result: Result<Symbol<PluginType>, _> = lib.get(b"create_plugin\0");
                        if let Ok(create_plugin) = result {
                            // Get the plugin object
                            let plugin_ptr = create_plugin();
                            let plugin: Box<dyn Plugin> = Box::from_raw(plugin_ptr); // Reclaim ownership

                            println!("Running update for Plugin: {}", plugin.name());

                            // Run the blocking update
                            if plugin.update(tx_plugin) {
                                println!("Update successful");
                            } else {
                                println!("Update failed");
                            }

                            // drop(tx_plugin);

                            // std::thread::sleep(std::time::Duration::from_millis(5000));
                        }
                    }

                    // wait for the plugin to finish
                };
            }
        });
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
    actions::set_about(app, &window);
    actions::set_preferences(app, &window, plugins.clone());
    actions::set_quit(app);

    // drop(tx);

    // Present window
    window.present();

    let ppbar_clone = plugin_progress_bar.clone();
    let tpbar_clone = total_progress_bar.clone();
    let apply_clone = apply_check_button.clone();
    let update_clone = update_button.clone();

    // This is called each time GTK is idle (i.e., not processing events).
    // It will run as often as possible but never blocks the main loop.
    // let mut total_progress: u32 = 0;
    // let mut total_status: String = String::new();

    // let total_progress_clone = total_progress.clone();
    //
    let settings = gio::Settings::new(config::APP_ID);

    // Load the enabled plugin(s)
    let plugins = settings.get::<Vec<String>>("plugins");

    let plugin_count = plugins.len();
    let mut plugin_index = 1;
    glib::idle_add_local(move || {
        // Try to receive a message. `try_recv` is non‑blocking.
        match rx.try_recv() {
            Ok(progress) => {
                let total_status = format!(
                    "Updating {} ({}/{})...",
                    progress.name, plugin_index, plugin_count
                );

                // Update the UI
                tpbar_clone.set_text(Some(&total_status));
                ppbar_clone.set_text(Some(&progress.status));
                println!("UI updated with: {:?}", progress);

                ppbar_clone.set_fraction(progress.progress as f64 / 100.0);

                // TODO: Calculate the total progress based on plugin(s) completed
                // ui.total_progress_bar.set_text(Some("Running uupd..."));
                // ui.total_progress_bar
                //     .set_fraction(p.progress as f64 / 100.0);

                if progress.progress == 100 {
                    apply_clone.set_sensitive(true);
                    update_clone.set_sensitive(true);
                    // total_progress += 1;
                    //
                    // If we're done updating the last plugin, update the UI
                    if plugin_index == plugin_count {
                        tpbar_clone.set_text(Some("Updates complete!"));
                        tpbar_clone.set_fraction(0.0);
                    } else {
                        plugin_index += 1;
                    }
                }

                // Return Continue if you want to keep listening,
                // or Stop if this idle callback should run only once.
                glib::ControlFlow::Continue
            }
            Err(flume::TryRecvError::Empty) => {
                // No messages, but there are sender(s) alive – keep the idle handler alive
                glib::ControlFlow::Continue
            }
            Err(flume::TryRecvError::Disconnected) => {
                // All senders dropped – no more work will come.
                println!("All senders disconnected. Stopping idle callback.");
                glib::ControlFlow::Break
            }
        }
    });

    return window;
}

// fn execute_command_async(mut ui: ui::UiModel) {
//     // Disable the update button and checkbox while running uupd
//     ui.apply_check_button.set_sensitive(false);
//     ui.update_button.set_sensitive(false);
//     ui.plugin_progress_bar.set_visible(true);
//     ui.total_progress_bar.set_visible(true);

//     let settings = gio::Settings::new(config::APP_ID);

//     // Load the enabled plugin(s)
//     let plugins = settings.get::<Vec<String>>("plugins");
//     ui.plugin_count = plugins.len() as u32;
//     // println!("Plugin count: {}", ui.plugin_count);
//     // let pool = ThreadPool::new(plugins.len());

//     // let (tx, rx) = mpsc::channel();
//     let (tx, rx) = flume::unbounded();
//     // let tx = Arc::new(Mutex::new(tx));

//     GLOBAL.with(|global| {
//         *global.borrow_mut() = Some((ui.clone(), rx));
//     });

//     for plugin in plugins {
//         let (tx, rx) = flume::unbounded();

//         // thread::spawn(move || {
//         //     (0..10).for_each(|i| {
//         //         let progress = Progress {
//         //             progress: i,
//         //             status: "Testing".to_string(),
//         //         };
//         //         tx.send(progress).unwrap();
//         //     })
//         // });

//         // let progress = rx.recv().unwrap();
//         // // let received: u32 = rx.iter().sum();
//         // println!("Received progress: {:?}", progress);

//         // Execute each plugin in a separate thread so we don't block the main thread
//         let handle = std::thread::spawn(move || {
//             unsafe {
//                 // Load the shared library
//                 if let Ok(lib) = Library::new(plugin) {
//                     // Instantiate the plugin
//                     let result: Result<Symbol<PluginType>, _> = lib.get(b"create_plugin\0");
//                     if let Ok(create_plugin) = result {
//                         // Get the plugin object
//                         let plugin_ptr = create_plugin();
//                         let plugin: Box<dyn Plugin> = Box::from_raw(plugin_ptr); // Reclaim ownership

//                         println!("Running update for Plugin: {}", plugin.name());

//                         let t = || {
//                             println!(
//                                 "closure tick called for thread {:?}",
//                                 std::thread::current().id()
//                             );
//                         };
//                         // pool.execute(|| {
//                         //     println!("Executing in thread {:?}", std::thread::current().id());
//                         // });

//                         // Run the blocking update
//                         if plugin.update(tx, tick) {
//                             println!("Update successful");
//                         } else {
//                             println!("Update failed");
//                         }

//                         // tick();
//                         std::thread::sleep(std::time::Duration::from_millis(5000));
//                     }
//                 }
//             };
//         });

//         // while let Ok(p) = rx.recv() {
//         //     println!("Received progress: {:?}", p);

//         //     // glib::source::idle_add(|| {
//         //     //     println!("Checking for new message");
//         //     //     check_for_new_message();
//         //     //     glib::ControlFlow::Break
//         //     // });

//         //     // ui.plugin_progress_bar.set_text(Some(&p.status));
//         //     // ui.plugin_progress_bar
//         //     //     .set_fraction(p.progress as f64 / 100.0);

//         //     // // TODO: Calculate the total progress based on plugin(s) completed
//         //     // ui.total_progress_bar.set_text(Some("Running uupd..."));
//         //     // ui.total_progress_bar
//         //     //     .set_fraction(p.progress as f64 / 100.0);

//         //     // let finished = p.progress == 100;

//         //     // // If the progress is complete, re-enable the disabled UI elements
//         //     // if finished {
//         //     //     ui.update_button.set_sensitive(true);
//         //     //     ui.apply_check_button.set_sensitive(true);

//         //     //     let reboot = ui.apply_check_button.is_active();

//         //     //     let msg = format!(
//         //     //         "Updates complete! {}",
//         //     //         if reboot { "Rebooting..." } else { "" }
//         //     //     );

//         //     //     // ui.plugin_progress_bar.set_pulse_step(0.0);

//         //     //     ui.total_progress_bar.set_text(Some(&msg));
//         //     //     ui.total_progress_bar.set_fraction(1.0);

//         //     //     if reboot && utils::check_reboot_needed() {
//         //     //         utils::reboot_system();
//         //     //     }
//         //     // }
//         // }

//         // println!("Thread ID: {:?}", handle.thread().id());

//         // println!("Main thread: Doing other work...");
//         // std::thread::sleep(std::time::Duration::from_millis(1000));

//         // println!("Main thread: Waiting for spawned thread to finish...");
//         // while !handle.is_finished() {
//         //     std::thread::sleep(std::time::Duration::from_millis(1000));

//         //     // tick();
//         //     // check_for_new_message();
//         // }
//         // // handle.join().unwrap();
//         // println!("Thread finished");
//     }
// }

// extern "Rust" fn tick() {
//     // Tell the UI thread to read from the channel
//     println!("tick from thread ID: {:?}", std::thread::current().id());

//     // glib::source::idle_add(|| {
//     //     println!("Checking for new message");
//     //     check_for_new_message();
//     //     glib::ControlFlow::Break
//     // });
// }

// // global variable to store the ui and an input channel
// // on the main thread only
// thread_local!(
//     static GLOBAL: RefCell<Option<(ui::UiModel, flume::Receiver<renovatio::Progress>)>> =
//         const { RefCell::new(None) };
// );

// // function to check if a new message has been passed through the
// // global receiver and, if so, add it to the UI.
// fn check_for_new_message() {
//     GLOBAL.with(|global| {
//         if let Some((ui, rx)) = &*global.borrow() {
//             // Receive the Progress struct

//             // In theory, we can use a loop to continuously receive messages from the channel
//             // so that we only need to call this function once per thread.
//             // while let Ok(p) = rx.try_recv() {
//             //     println!("Receiving on thread ID: {:?}", std::thread::current().id());
//             //     println!("progress: {:?}", p);

//             //     // Plugin progress goes to the plugin_progress_bar
//             //     ui.plugin_progress_bar.set_text(Some(&p.status));
//             //     ui.plugin_progress_bar
//             //         .set_fraction(p.progress as f64 / 100.0);

//             //     // TODO: Calculate the total progress based on plugin(s) completed
//             //     ui.total_progress_bar.set_text(Some("Running uupd..."));
//             //     ui.total_progress_bar
//             //         .set_fraction(p.progress as f64 / 100.0);

//             //     let finished = p.progress == 100;

//             //     // If the progress is complete, re-enable the disabled UI elements
//             //     if finished {
//             //         ui.update_button.set_sensitive(true);
//             //         ui.apply_check_button.set_sensitive(true);

//             //         let reboot = ui.apply_check_button.is_active();

//             //         let msg = format!(
//             //             "Updates complete! {}",
//             //             if reboot { "Rebooting..." } else { "" }
//             //         );

//             //         // ui.plugin_progress_bar.set_pulse_step(0.0);

//             //         ui.total_progress_bar.set_text(Some(&msg));
//             //         ui.total_progress_bar.set_fraction(1.0);

//             //         if reboot && utils::check_reboot_needed() {
//             //             utils::reboot_system();
//             //         }
//             //     }
//             // }
//             println!("Receiving on thread ID: {:?}", std::thread::current().id());

//             let res = rx.recv();
//             if let Err(e) = res {
//                 println!("Error receiving progress: {}", e);
//                 return;
//             }

//             let p: renovatio::Progress = res.unwrap();
//             println!("progress: {:?}", p);

//             // Plugin progress goes to the plugin_progress_bar
//             ui.plugin_progress_bar.set_text(Some(&p.status));
//             ui.plugin_progress_bar
//                 .set_fraction(p.progress as f64 / 100.0);

//             // TODO: Calculate the total progress based on plugin(s) completed
//             ui.total_progress_bar.set_text(Some("Running uupd..."));
//             ui.total_progress_bar
//                 .set_fraction(p.progress as f64 / 100.0);

//             let finished = p.progress == 100;

//             // If the progress is complete, re-enable the disabled UI elements
//             if finished {
//                 ui.update_button.set_sensitive(true);
//                 ui.apply_check_button.set_sensitive(true);

//                 let reboot = ui.apply_check_button.is_active();

//                 let msg = format!(
//                     "Updates complete! {}",
//                     if reboot { "Rebooting..." } else { "" }
//                 );

//                 // ui.plugin_progress_bar.set_pulse_step(0.0);

//                 ui.total_progress_bar.set_text(Some(&msg));
//                 ui.total_progress_bar.set_fraction(1.0);

//                 if reboot && utils::check_reboot_needed() {
//                     utils::reboot_system();
//                 }
//             }
//         }
//     });
// }
