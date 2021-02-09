mod apply_revealer;
mod header;
mod root_stack;

extern crate gtk;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use apply_revealer::ApplyRevealer;
use daemon::daemon_connection::DaemonConnection;
use daemon::gpu_controller::GpuStats;
use daemon::DaemonError;
use gtk::*;

use header::Header;
use root_stack::RootStack;

#[derive(Clone)]
pub struct App {
    pub window: Window,
    pub header: Header,
    root_stack: RootStack,
    apply_revealer: ApplyRevealer,
    daemon_connection: DaemonConnection,
}

impl App {
    pub fn new(daemon_connection: DaemonConnection) -> Self {
        let window = Window::new(WindowType::Toplevel);

        let header = Header::new();

        window.set_titlebar(Some(&header.container));
        window.set_title("LACT");

        window.set_default_size(500, 600);

        window.connect_delete_event(move |_, _| {
            main_quit();
            Inhibit(false)
        });

        let root_stack = RootStack::new();

        header.set_switcher_stack(&root_stack.container);

        let root_box = Box::new(Orientation::Vertical, 5);

        root_box.add(&root_stack.container);

        let apply_revealer = ApplyRevealer::new();

        root_box.add(&apply_revealer.container);

        window.add(&root_box);

        App {
            window,
            header,
            root_stack,
            apply_revealer,
            daemon_connection,
        }
    }

    pub fn run(&self) -> Result<(), DaemonError> {
        let current_gpu_id = Arc::new(AtomicU32::new(0));

        {
            let current_gpu_id = current_gpu_id.clone();
            let app = self.clone();

            self.header.connect_gpu_selection_changed(move |gpu_id| {
                app.set_info(gpu_id);
                current_gpu_id.store(gpu_id, Ordering::SeqCst);
            });
        }

        let gpus = self.daemon_connection.get_gpus()?;

        self.header.set_gpus(gpus);

        // Show apply button on setting changes
        {
            let apply_revealer = self.apply_revealer.clone();
            self.root_stack
                .thermals_page
                .connect_settings_changed(move || {
                    apply_revealer.show();
                });
        }

        // Apply settings
        {
            let current_gpu_id = current_gpu_id.clone();
            let app = self.clone();

            self.apply_revealer.connect_apply_button_clicked(move || {
                let gpu_id = current_gpu_id.load(Ordering::SeqCst);

                let thermals_settings = app.root_stack.thermals_page.get_thermals_settings();

                if thermals_settings.automatic_fan_control_enabled {
                    app.daemon_connection
                        .stop_fan_control(gpu_id)
                        .expect("Failed to top fan control");
                } else {
                    app.daemon_connection
                        .start_fan_control(gpu_id)
                        .expect("Failed to start fan control");
                }

                app.set_info(gpu_id);
            });
        }

        self.start_stats_update_loop(current_gpu_id.clone());

        self.window.show_all();

        Ok(gtk::main())
    }

    fn set_info(&self, gpu_id: u32) {
        let gpu_info = self.daemon_connection.get_gpu_info(gpu_id).unwrap();
        self.root_stack.info_page.set_info(gpu_info);

        match self.daemon_connection.get_fan_control(gpu_id) {
            Ok(fan_control_info) => self
                .root_stack
                .thermals_page
                .set_ventilation_info(fan_control_info),
            Err(_) => self.root_stack.thermals_page.hide_fan_controls(),
        }
        
        self.apply_revealer.hide();
    }

    fn start_stats_update_loop(&self, current_gpu_id: Arc<AtomicU32>) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        {
            let daemon_connection = self.daemon_connection.clone();

            thread::spawn(move || loop {
                let gpu_id = current_gpu_id.load(Ordering::SeqCst);

                if let Ok(stats) = daemon_connection.get_gpu_stats(gpu_id) {
                    sender.send(GuiUpdateMsg::GpuStats(stats)).unwrap();
                }

                thread::sleep(Duration::from_millis(500));
            });
        }

        {
            let thermals_page = self.root_stack.thermals_page.clone();
            let oc_page = self.root_stack.oc_page.clone();

            receiver.attach(None, move |msg| {
                match msg {
                    GuiUpdateMsg::GpuStats(stats) => {
                        thermals_page.set_thermals_info(&stats);
                        oc_page.set_stats(&stats);
                    } /*GuiUpdateMsg::FanControlInfo(fan_control_info) => {
                          thermals_page.set_ventilation_info(fan_control_info)
                      }*/
                }

                glib::Continue(true)
            });
        }
    }
}

enum GuiUpdateMsg {
    // FanControlInfo(FanControlInfo),
    GpuStats(GpuStats),
}
