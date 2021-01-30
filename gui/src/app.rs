mod header;
mod root_stack;

extern crate gtk;

use daemon::daemon_connection::DaemonConnection;
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
        
        window.set_size_request(500, 600);

        window.connect_delete_event(move |_, _| {
            main_quit();
            Inhibit(false)
        });

        let root_stack = RootStack::new();

        header.set_switcher_stack(&root_stack.container);

        window.add(&root_stack.container);
        
        App { window, header, root_stack }
    }
    
    pub fn run(&self, daemon_connection: DaemonConnection) {
        self.window.show_all();
        
        let gpus = daemon_connection.get_gpus().unwrap();
        let gpu_info = daemon_connection.get_gpu_info(*gpus.iter().next().unwrap().0).unwrap();
        self.root_stack.info_page.set_info(gpu_info);

        gtk::main();
    }
}