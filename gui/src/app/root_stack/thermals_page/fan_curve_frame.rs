use gtk::*;

#[derive(Clone)]
pub struct FanCurveFrame {
    pub container: Frame,
}

impl FanCurveFrame {
    pub fn new() -> Self {
        let container = Frame::new(Some("Fan Curve"));

        container.set_shadow_type(ShadowType::None);

        Self { container }
    }
}
