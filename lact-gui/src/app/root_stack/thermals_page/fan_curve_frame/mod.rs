mod point_adjustment;

use self::point_adjustment::PointAdjustment;
use glib::clone;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{default_fan_curve, FanCurveMap};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct FanCurveFrame {
    #[cfg(feature = "adw")]
    pub container: Box,

    #[cfg(not(feature = "adw"))]
    pub container: Frame,

    curve_container: ScrolledWindow,
    points: Rc<RefCell<Vec<PointAdjustment>>>,
}

impl FanCurveFrame {
    pub fn new() -> Self {
        let root_box = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["card"])
            .height_request(450)
            .build();

        let hbox = Box::new(Orientation::Horizontal, 6);

        let curve_container = ScrolledWindow::builder()
            .vscrollbar_policy(PolicyType::Never)
            .build();

        hbox.append(&curve_container);

        let buttons_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true)
            .spacing(12)
            .margin_bottom(12)
            .halign(Align::Center)
            .build();

        let add_button = Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(["circular"])
            .tooltip_text("Add point")
            .build();
        let remove_button = Button::builder()
            .icon_name("list-remove-symbolic")
            .css_classes(["circular"])
            .tooltip_text("Remove last point")
            .build();
        let default_button = Button::builder()
            .css_classes(["circular"])
            .child(
                &Label::builder()
                    .label("Reset")
                    .margin_start(12)
                    .margin_end(12)
                    .build(),
            )
            .build();

        buttons_box.append(&remove_button);
        buttons_box.append(&default_button);
        buttons_box.append(&add_button);

        root_box.append(&hbox);
        root_box.append(
            &Separator::builder()
                .orientation(Orientation::Horizontal)
                .margin_bottom(12)
                .build(),
        );
        root_box.append(&buttons_box);

        let points = Rc::new(RefCell::new(Vec::new()));

        let curve_frame = Self {
            #[cfg(feature = "adw")]
            container: root_box,

            #[cfg(not(feature = "adw"))]
            container: Frame::builder()
                .css_classes(["view"])
                .child(&root_box)
                .build(),

            curve_container,
            points,
        };

        default_button.connect_clicked(clone!(@strong curve_frame => move |_| {
            let curve = default_fan_curve();
            curve_frame.set_curve(&curve);
        }));

        add_button.connect_clicked(clone!(@strong curve_frame  => move |_| {
            curve_frame.add_point();
        }));

        remove_button.connect_clicked(clone!(@strong curve_frame  => move |_| {
            curve_frame.remove_point();
        }));

        curve_frame
    }

    fn add_point(&self) {
        let mut curve = self.get_curve();
        if let Some((temperature, ratio)) = curve.iter().last() {
            curve.insert(temperature + 5, *ratio);
            self.set_curve(&curve);
        } else {
            curve.insert(50, 0.5);
            self.set_curve(&curve);
        }
    }

    fn remove_point(&self) {
        let mut curve = self.get_curve();
        curve.pop_last();
        self.set_curve(&curve);
    }

    fn notify_changed(&self) {
        if let Some(point) = self.points.borrow().first() {
            point.ratio.emit_by_name::<()>("value-changed", &[]);
        }
    }

    pub fn set_curve(&self, curve: &FanCurveMap) {
        // Notify that the values were changed when the entire curve is overwritten, e.g. when resetting to default
        self.notify_changed();

        let points_container = Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_bottom(12)
            .vexpand(true)
            .build();

        let mut adjustments = Vec::with_capacity(curve.len());

        for (temperature, ratio) in curve {
            let adjustment = PointAdjustment::new(&points_container, *ratio, *temperature);
            adjustments.push(adjustment);
        }

        self.points.replace(adjustments);
        self.curve_container.set_child(Some(&points_container));
    }

    pub fn get_curve(&self) -> FanCurveMap {
        let mut curve = BTreeMap::new();

        let points = self.points.borrow();
        for point in &*points {
            let temperature = point.temperature.value() as i32;
            let ratio = point.ratio.value() as f32;
            curve.insert(temperature, ratio);
        }

        curve
    }

    pub fn connect_adjusted<F: Fn() + 'static + Clone>(&self, f: F) {
        let closure = clone!(@strong f => move |_: &Adjustment| {
            f();
        });

        for point in &*self.points.borrow() {
            point.ratio.connect_value_changed(closure.clone());
            point.temperature.connect_value_changed(closure.clone());
        }
    }
}

#[cfg(all(test, feature = "gtk-tests"))]
mod tests {
    use super::FanCurveFrame;
    use lact_client::schema::default_fan_curve;
    use pretty_assertions::assert_eq;

    #[test]
    fn set_get_curve() {
        gtk::init().unwrap();

        let curve = default_fan_curve();
        let frame = FanCurveFrame::new();
        frame.set_curve(&curve);
        let received_curve = frame.get_curve();
        assert_eq!(received_curve, curve);
    }
}
