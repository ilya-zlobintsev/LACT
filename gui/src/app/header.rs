use std::env;

use gtk::*;

pub struct Header {
    pub container: HeaderBar,
    switcher: StackSwitcher,
}

impl Header {
    pub fn new() -> Self {
        let container = HeaderBar::new();

        // container.set_title(Some("LACT"));

        if env::var("XDG_CURRENT_DESKTOP") == Ok("GNOME".to_string()) {
            container.set_show_close_button(true);
        }
        
        let switcher = StackSwitcher::new();
        
        container.pack_start(&switcher);
        
        Header { container, switcher }
    } 
    
    pub fn set_switcher_stack(&self, stack: &Stack) {
        self.switcher.set_stack(Some(stack));
    }
}