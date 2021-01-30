mod header;
mod root_stack;

extern crate gtk;

use daemon::{daemon_connection::DaemonConnection, DaemonError};
use gtk::*;

use header::Header;
use root_stack::RootStack;

pub struct App {
    pub window: Window,
    pub header: Header,
    root_stack: RootStack,
}

impl App {
    pub fn new() -> Self {
        let window = Window::new(WindowType::Toplevel);

        let header = Header::new();

        window.set_titlebar(Some(&header.container));

        window.set_title("LACT");
        // window.set_wmclass("lact", "LACT");

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
        }
    }

    pub fn run(&self, daemon_connection: DaemonConnection) -> Result<(), DaemonError> {
        let root_stack = self.root_stack.clone();

        self.header.connect_gpu_selection_changed(move |gpu_id| {
            let gpu_info = daemon_connection.get_gpu_info(gpu_id).unwrap();
            root_stack.info_page.set_info(gpu_info);
        });

        let gpus = daemon_connection.get_gpus()?;

        self.header.set_gpus(gpus);

        self.window.show_all();

        Ok(gtk::main())
    }
}
