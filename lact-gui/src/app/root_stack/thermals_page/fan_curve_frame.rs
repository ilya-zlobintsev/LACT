use gtk::prelude::*;
use gtk::*;
use lact_client::schema::FanCurveMap;
use std::cell::RefCell;
use tracing::debug;

#[derive(Clone)]
pub struct FanCurveFrame {
    pub container: Frame,
    points: RefCell<Vec<Adjustment>>,
}

impl FanCurveFrame {
    pub fn new() -> Self {
        let root_container = Frame::new(Some("Fan Curve"));

        root_container.set_margin_start(10);
        root_container.set_margin_end(10);
        root_container.set_margin_bottom(10);
        root_container.set_margin_top(10);

        let points = RefCell::new(Vec::new());

        Self {
            container: root_container,
            points,
        }
    }

    pub fn set_curve(&self, curve: &FanCurveMap) {
        let points_container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .vexpand(true)
            .build();

        let mut adjustments = Vec::with_capacity(curve.len());

        for (temperature, ratio) in curve {
            let adjustment = point_adjustment(&points_container, *ratio, *temperature);
            adjustments.push(adjustment);
        }

        self.points.replace(adjustments);
        self.container.set_child(Some(&points_container));
    }

    /*pub fn get_curve(&self) -> BTreeMap<i64, f64> {
        let mut curve = BTreeMap::new();

        curve.insert(20, self.adjustment_1.value());
        curve.insert(40, self.adjustment_2.value());
        curve.insert(60, self.adjustment_3.value());
        curve.insert(80, self.adjustment_4.value());
        curve.insert(100, self.adjustment_5.value());

        curve
    }*/

    // pub fn show(&self) {
    //     debug!("Manual fan control enaged, showing fan curve");
    //     self.container.set_visible(true);
    // }

    // pub fn hide(&self) {
    //     debug!("Manual fan control disenaged, hiding fan curve");
    //     self.container.set_visible(false);
    // }

    /*pub fn connect_adjusted<F: Fn() + 'static + Clone>(&self, f: F) {
        let adjustments = [
            &self.adjustment_1,
            &self.adjustment_2,
            &self.adjustment_3,
            &self.adjustment_4,
            &self.adjustment_5,
        ];

        for adj in adjustments.iter() {
            let f = f.clone();
            adj.connect_value_changed(move |_| {
                f();
            });
        }
    }*/
}

fn point_adjustment(parent: &Box, ratio: f32, temperature: i32) -> Adjustment {
    let container = Box::new(Orientation::Vertical, 5);

    let adjustment = Adjustment::new(ratio.into(), 0.0, 1.0, 0.01, 0.05, 0.05);
    let scale = Scale::builder()
        .orientation(Orientation::Vertical)
        .adjustment(&adjustment)
        .hexpand(true)
        .vexpand(true)
        .inverted(true)
        .build();
    container.append(&scale);

    let temp_label = Label::new(Some(&temperature.to_string()));
    container.append(&temp_label);

    parent.append(&container);
    adjustment
}
