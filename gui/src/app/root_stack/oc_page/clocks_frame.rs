use daemon::gpu_controller::ClocksTable;
use gtk::*;

pub struct ClocksSettings {
    pub gpu_clock: i64,
    pub vram_clock: i64,
    pub gpu_voltage: i64,
}

#[derive(Clone)]
pub struct ClocksFrame {
    pub container: Frame,
    gpu_clock_adjustment: Adjustment,
    gpu_voltage_adjustment: Adjustment,
    vram_clock_adjustment: Adjustment,
    apply_button: Button,
}

impl ClocksFrame {
    pub fn new() -> Self {
        let container = Frame::new(None);

        container.set_margin_start(10);
        container.set_margin_end(10);

        container.set_shadow_type(ShadowType::None);

        container.set_label_widget(Some(&{
            let label = Label::new(None);
            label.set_markup("<span font_desc='11'><b>Maximum Clocks</b></span>");
            label
        }));
        container.set_label_align(0.2, 0.0);

        let gpu_clock_adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);

        let gpu_voltage_adjustment = Adjustment::new(1.0, 0.0, 0.0, 0.05, 0.0, 0.0);

        let vram_clock_adjustment = Adjustment::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);

        let root_grid = Grid::new();

        root_grid.set_row_spacing(5);
        root_grid.set_column_spacing(10);

        {
            let gpu_clock_scale = Scale::new(Orientation::Horizontal, Some(&gpu_clock_adjustment));

            gpu_clock_scale.set_hexpand(true); // Affects the grid column and all scales

            gpu_clock_scale.set_value_pos(PositionType::Right);

            root_grid.attach(&gpu_clock_scale, 1, 0, 1, 1);

            root_grid.attach_next_to(
                &Label::new(Some("GPU Clock (MHz)")),
                Some(&gpu_clock_scale),
                PositionType::Left,
                1,
                1,
            );

            let gpu_voltage_scale =
                Scale::new(Orientation::Horizontal, Some(&gpu_voltage_adjustment));

            gpu_voltage_scale.set_value_pos(PositionType::Right);

            gpu_voltage_scale.set_digits(3);
            gpu_voltage_scale.set_round_digits(3);

            root_grid.attach(&gpu_voltage_scale, 1, 1, 1, 1);

            root_grid.attach_next_to(
                &Label::new(Some("GPU Voltage (V)")),
                Some(&gpu_voltage_scale),
                PositionType::Left,
                1,
                1,
            );

            let vram_clock_scale =
                Scale::new(Orientation::Horizontal, Some(&vram_clock_adjustment));

            vram_clock_scale.set_value_pos(PositionType::Right);

            root_grid.attach(&vram_clock_scale, 1, 2, 1, 1);

            root_grid.attach_next_to(
                &Label::new(Some("VRAM Clock (MHz)")),
                Some(&vram_clock_scale),
                PositionType::Left,
                1,
                1,
            );
        }

        let apply_button = Button::new();

        {
            apply_button.set_label("Reset");

            root_grid.attach(&apply_button, 0, 3, 2, 1);

            container.add(&root_grid);
        }

        Self {
            container,
            gpu_clock_adjustment,
            gpu_voltage_adjustment,
            vram_clock_adjustment,
            apply_button,
        }
    }

    pub fn get_visibility(&self) -> bool {
        self.container.get_visible()
    }

    pub fn set_clocks(&self, clocks_table: &ClocksTable) {
        match clocks_table {
            ClocksTable::Old(clocks_table) => {
                self.gpu_clock_adjustment
                    .set_lower(clocks_table.gpu_clocks_range.0 as f64);
                self.gpu_clock_adjustment
                    .set_upper(clocks_table.gpu_clocks_range.1 as f64);

                self.gpu_voltage_adjustment
                    .set_lower(clocks_table.voltage_range.0 as f64 / 1000.0);
                self.gpu_voltage_adjustment
                    .set_upper(clocks_table.voltage_range.1 as f64 / 1000.0);

                self.vram_clock_adjustment
                    .set_lower(clocks_table.mem_clocks_range.0 as f64);
                self.vram_clock_adjustment
                    .set_upper(clocks_table.mem_clocks_range.1 as f64);

                let (gpu_clockspeed, gpu_voltage) =
                    clocks_table.gpu_power_levels.iter().next_back().unwrap().1;

                self.gpu_clock_adjustment.set_value(*gpu_clockspeed as f64);

                self.gpu_voltage_adjustment
                    .set_value(*gpu_voltage as f64 / 1000.0);

                let (vram_clockspeed, _) =
                    clocks_table.mem_power_levels.iter().next_back().unwrap().1;

                self.vram_clock_adjustment
                    .set_value(*vram_clockspeed as f64);
            }
            ClocksTable::New(clocks_table) => {
                self.gpu_clock_adjustment
                    .set_lower(clocks_table.gpu_clocks_range.0 as f64);
                self.gpu_clock_adjustment
                    .set_upper(clocks_table.gpu_clocks_range.1 as f64);

                /* self.gpu_voltage_adjustment
                    .set_lower(clocks_table.voltage_range.0 as f64 / 1000.0);
                 self.gpu_voltage_adjustment
                    .set_upper(clocks_table.voltage_range.1 as f64 / 1000.0);*/

                self.vram_clock_adjustment
                    .set_lower(clocks_table.mem_clocks_range.0 as f64);
                self.vram_clock_adjustment
                    .set_upper(clocks_table.mem_clocks_range.1 as f64);


                self.gpu_clock_adjustment.set_value(clocks_table.current_gpu_clocks.1 as f64);

                // self.gpu_voltage_adjustment
                    // .set_value(*clocks_table.gpu_voltage as f64 / 1000.0);

                self.vram_clock_adjustment
                    .set_value(clocks_table.current_max_mem_clock as f64);
            }
        }
    }

    pub fn get_settings(&self) -> ClocksSettings {
        let gpu_clock = self.gpu_clock_adjustment.get_value() as i64;

        let vram_clock = self.vram_clock_adjustment.get_value() as i64;

        let gpu_voltage = (self.gpu_voltage_adjustment.get_value() * 1000.0) as i64;

        ClocksSettings {
            gpu_clock,
            vram_clock,
            gpu_voltage,
        }
    }

    pub fn connect_clocks_reset<F: Fn() + 'static + Clone>(&self, f: F) {
        self.apply_button.connect_clicked(move |_| {
            f();
        });
    }

    pub fn connect_clocks_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        {
            let f = f.clone();
            self.gpu_clock_adjustment.connect_value_changed(move |_| {
                f();
            });
        }
        {
            let f = f.clone();
            self.vram_clock_adjustment.connect_value_changed(move |_| {
                f();
            });
        }
        {
            self.gpu_voltage_adjustment.connect_value_changed(move |_| {
                f();
            });
        }
    }

    pub fn hide(&self) {
        self.container.set_visible(false);
    }

    pub fn show(&self) {
        self.container.set_visible(true);
    }
}
