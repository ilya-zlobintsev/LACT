use gtk::Frame;

#[derive(Clone)]
pub struct ClocksFrame {
    pub container: Frame,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = Frame::new(None);

        Self { container }
    }
}
