mod header;
mod root_stack;

extern crate gtk;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use daemon::{daemon_connection::DaemonConnection, DaemonError};
use gtk::*;

use header::Header;
use root_stack::RootStack;

pub struct App {
    pub window: Window,
    pub header: Header,
    root_stack: RootStack,
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

        window.add(&root_stack.container);

        App {
            window,
            header,
            root_stack,
            daemon_connection,
        }
    }

    pub fn run(&self) -> Result<(), DaemonError> {
        let current_gpu_id = Arc::new(AtomicU32::new(0));

        let root_stack = self.root_stack.clone();

        {
            let current_gpu_id = current_gpu_id.clone();
            let daemon_connection = self.daemon_connection.clone();

            self.header.connect_gpu_selection_changed(move |gpu_id| {
                let gpu_info = daemon_connection.get_gpu_info(gpu_id).unwrap();
                root_stack.info_page.set_info(gpu_info);

                current_gpu_id.store(gpu_id, Ordering::SeqCst);
            });
        }

        self.start_stats_update_loop(current_gpu_id.clone());

        let gpus = self.daemon_connection.get_gpus()?;

        self.header.set_gpus(gpus);

        self.window.show_all();

        Ok(gtk::main())
    }

    fn start_stats_update_loop(&self, current_gpu_id: Arc<AtomicU32>) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let daemon_connection = self.daemon_connection.clone();

        thread::spawn(move || loop {
            let gpu_id = current_gpu_id.load(Ordering::SeqCst);

            if let Ok(stats) = daemon_connection.get_gpu_stats(gpu_id) {
                sender.send(stats).unwrap();
            }

            if let Ok(fan_control) = daemon_connection.get_fan_control(gpu_id) {
                println!("{:?}", fan_control);
            }

            thread::sleep(Duration::from_millis(500));
        });

        let thermals_page = self.root_stack.thermals_page.clone();

        receiver.attach(None, move |stats| {
            thermals_page.set_thermals_info(&stats);

            glib::Continue(true)
        });
    }
}
