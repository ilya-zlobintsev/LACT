use std::collections::BTreeMap;

use gtk::*;

#[derive(Clone)]
pub struct FanCurveFrame {
    pub container: Frame,
    adjustment_1: Adjustment,
    adjustment_2: Adjustment,
    adjustment_3: Adjustment,
    adjustment_4: Adjustment,
    adjustment_5: Adjustment,
}

impl FanCurveFrame {
    pub fn new() -> Self {
        let container = Frame::new(Some("Fan Curve"));

        container.set_margin_start(10);
        container.set_margin_end(10);
        container.set_margin_bottom(10);
        container.set_margin_top(10);

        container.set_label_align(0.35, 0.5);

        // container.set_shadow_type(ShadowType::None);
        //
        let root_grid = Grid::new();

        // PWM Percentage Labels
        {
            root_grid.attach(
                &{
                    let label = Label::new(Some("PWM %"));
                    label.set_angle(90.0);
                    label.set_vexpand(true); // This expands the entire top section of the grid, including the scales
                    label
                },
                0,
                0,
                1,
                5,
            );

            root_grid.attach(
                &{
                    let label = Label::new(Some("0"));
                    label.set_angle(90.0);
                    label
                },
                1,
                4,
                1,
                1,
            );
            root_grid.attach(
                &{
                    let label = Label::new(Some("25"));
                    label.set_angle(90.0);
                    label
                },
                1,
                3,
                1,
                1,
            );
            root_grid.attach(
                &{
                    let label = Label::new(Some("50"));
                    label.set_angle(90.0);
                    label
                },
                1,
                2,
                1,
                1,
            );
            root_grid.attach(
                &{
                    let label = Label::new(Some("75"));
                    label.set_angle(90.0);
                    label
                },
                1,
                1,
                1,
                1,
            );
            root_grid.attach(
                &{
                    let label = Label::new(Some("100"));
                    label.set_angle(90.0);
                    label
                },
                1,
                0,
                1,
                1,
            );
        }

        // Temperature threshold labels
        {
            root_grid.attach(
                &{
                    let label = Label::new(Some("Temperature °C"));
                    label.set_hexpand(true);
                    label
                },
                2,
                7,
                5,
                1,
            );

            root_grid.attach(&Label::new(Some("20")), 2, 6, 1, 1);
            root_grid.attach(&Label::new(Some("40")), 3, 6, 1, 1);
            root_grid.attach(&Label::new(Some("60")), 4, 6, 1, 1);
            root_grid.attach(&Label::new(Some("80")), 5, 6, 1, 1);
            root_grid.attach(&Label::new(Some("100")), 6, 6, 1, 1);
        }

        // The actual adjustments
        let adjustment_1 = Adjustment::new(0.0, 0.0, 100.0, 1.0, 0.0, 0.0); // 20 °C
        let adjustment_2 = Adjustment::new(0.0, 0.0, 100.0, 1.0, 0.0, 0.0); // 40 °C
        let adjustment_3 = Adjustment::new(0.0, 0.0, 100.0, 1.0, 0.0, 0.0); // 60 °C
        let adjustment_4 = Adjustment::new(0.0, 0.0, 100.0, 1.0, 0.0, 0.0); // 80 °C
        let adjustment_5 = Adjustment::new(0.0, 0.0, 100.0, 1.0, 0.0, 0.0); // 100 °C

        // Scales for the adjustments
        {
            let adjustments = [
                &adjustment_1,
                &adjustment_2,
                &adjustment_3,
                &adjustment_4,
                &adjustment_5,
            ];

            for i in 0..adjustments.len() {
                let adj = adjustments[i];

                root_grid.attach(
                    &{
                        let scale = Scale::new(Orientation::Vertical, Some(adj));
                        scale.set_draw_value(false);
                        scale.set_inverted(true);
                        scale
                    },
                    i as i32 + 2,
                    0,
                    1,
                    5,
                );
            }
        }

        container.add(&root_grid);

        Self {
            container,
            adjustment_1,
            adjustment_2,
            adjustment_3,
            adjustment_4,
            adjustment_5,
        }
    }

    pub fn set_curve(&self, curve: &BTreeMap<i64, f64>) {
        self.adjustment_1.set_value(*curve.get(&20).unwrap());
        self.adjustment_2.set_value(*curve.get(&40).unwrap());
        self.adjustment_3.set_value(*curve.get(&60).unwrap());
        self.adjustment_4.set_value(*curve.get(&80).unwrap());
        self.adjustment_5.set_value(*curve.get(&100).unwrap());
    }

    pub fn get_curve(&self) -> BTreeMap<i64, f64> {
        let mut curve = BTreeMap::new();

        curve.insert(20, self.adjustment_1.get_value());
        curve.insert(40, self.adjustment_2.get_value());
        curve.insert(60, self.adjustment_3.get_value());
        curve.insert(80, self.adjustment_4.get_value());
        curve.insert(100, self.adjustment_5.get_value());

        curve
    }

    pub fn show(&self) {
        log::info!("Manual fan control enaged, showing fan curve");
        self.container.set_visible(true);
    }

    pub fn hide(&self) {
        log::info!("Manual fan control disenaged, hiding fan curve");
        self.container.set_visible(false);
    }

    pub fn connect_adjusted<F: Fn() + 'static + Clone>(&self, f: F) {
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
    }
}
