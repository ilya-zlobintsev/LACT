use super::render_thread::{RenderRequest, RenderThread};
use super::PlotColorScheme;
use crate::app::graphs_window::stat::StatType;
use crate::app::graphs_window::stat::StatsData;
use glib::Properties;
use gtk::gdk::MemoryTexture;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::Cell;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::Plot)]
pub struct Plot {
    #[property(get, set)]
    title: RefCell<String>,
    #[property(set)]
    dirty: Cell<bool>,
    #[property(get, set)]
    print_extra_info: Cell<bool>,
    pub(super) render_thread: RenderThread,
    #[property(get, set)]
    time_period_seconds: Cell<i64>,
    pub(super) stats: RefCell<Vec<StatType>>,
    pub(super) data: RefCell<Arc<RwLock<StatsData>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Plot {
    const NAME: &'static str = "Plot";
    type Type = super::Plot;
    type ParentType = gtk::Widget;
}

#[glib::derived_properties]
impl ObjectImpl for Plot {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.set_height_request(250);
        obj.set_hexpand(true);
        obj.set_vexpand(true);
    }
}

impl WidgetImpl for Plot {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let width = self.obj().width() as u32;
        let height = self.obj().height() as u32;
        let scale_factor = self.obj().scale_factor();

        let style_context = self.obj().style_context();
        let colors = PlotColorScheme::from_context(&style_context).unwrap_or_default();

        if width == 0 || height == 0 {
            return;
        }

        let last_texture = self.get_last_texture();
        let size_changed = last_texture
            .as_ref()
            .map(|texture| (texture.width() as u32, texture.height() as u32) != (width, height))
            .unwrap_or(true);

        if self.dirty.replace(false) || size_changed {
            self.render_thread.replace_render_request(RenderRequest {
                data: self.data.borrow().clone(),
                stats: self.stats.borrow().clone(),
                width,
                height,
                scale_factor,
                colors,
                title: self.title.borrow().clone(),
                print_extra_info: self.print_extra_info.get(),
                time_period_seconds: self.time_period_seconds.get(),
            });
        }

        // Rendering is always behind by at least one frame, but it's not an issue
        if let Some(texture) = last_texture {
            let bounds = gtk::graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
            // Uses by default Trillinear texture filtering, which is quite good at 4x supersampling
            snapshot.append_texture(&texture, &bounds);
        }
    }
}

impl Plot {
    pub fn get_last_texture(&self) -> Option<MemoryTexture> {
        self.render_thread.get_last_texture()
    }
}
#[cfg(feature = "bench")]
mod benches {
    use crate::app::graphs_window::{
        plot::{
            render_thread::{process_request, RenderRequest},
            PlotColorScheme,
        },
        stat::{StatType, StatsData},
    };
    use amdgpu_sysfs::{gpu_handle::PerformanceLevel, hw_mon::Temperature};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use divan::{counter::ItemsCount, Bencher};
    use lact_schema::{
        ClockspeedStats, DeviceStats, FanStats, PmfwInfo, PowerStats, TemperatureEntry,
        VoltageStats, VramStats,
    };
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex, RwLock},
    };

    #[divan::bench(sample_size = 10)]
    fn render_plot(bencher: Bencher) {
        let last_texture = &Mutex::new(None);

        bencher
            .with_inputs(sample_plot_data)
            .input_counter(|_| ItemsCount::new(1usize))
            .bench_values(|data| {
                let request = RenderRequest {
                    title: "bench render".into(),
                    colors: PlotColorScheme::default(),
                    data,
                    width: 1920,
                    height: 1080,
                    scale_factor: 1,
                    time_period_seconds: 60,
                    print_extra_info: false,
                    stats: vec![
                        StatType::GpuClock,
                        StatType::GpuTargetClock,
                        StatType::VramClock,
                        StatType::GpuVoltage,
                    ],
                };

                process_request(request, last_texture)
            });
    }

    fn sample_plot_data() -> Arc<RwLock<StatsData>> {
        let mut data = StatsData::default();

        // Simulate 1 minute plot with 4 values per second
        for sec in 0..60 {
            for milli in [0, 250, 500, 750] {
                let datetime = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    NaiveTime::from_hms_milli_opt(0, 0, sec, milli).unwrap(),
                );

                let stats = DeviceStats {
                    busy_percent: Some(3),
                    clockspeed: ClockspeedStats {
                        gpu_clockspeed: Some(500),
                        vram_clockspeed: Some(1000),
                        target_gpu_clockspeed: None,
                        sensors: HashMap::new(),
                    },
                    core_power_state: Some(0),
                    fan: FanStats {
                        control_enabled: false,
                        pmfw_info: PmfwInfo::default(),
                        pwm_current: Some(0),
                        pwm_max: Some(255),
                        pwm_min: Some(0),
                        speed_current: Some(0),
                        speed_max: Some(3400),
                        speed_min: Some(0),
                        ..Default::default()
                    },
                    memory_power_state: Some(3),
                    pcie_power_state: Some(1),
                    performance_level: Some(PerformanceLevel::Auto),
                    power: PowerStats {
                        average: Some(36.0),
                        cap_current: Some(289.0),
                        cap_default: Some(289.0),
                        cap_max: Some(332.0),
                        cap_min: Some(0.0),
                        current: None,
                    },
                    temps: HashMap::from([(
                        "edge".to_owned(),
                        TemperatureEntry {
                            value: Temperature {
                                crit: Some(100.0),
                                crit_hyst: None,
                                current: Some(56.0),
                            },
                            display_only: false,
                        },
                    )]),
                    voltage: VoltageStats::default(),
                    vram: VramStats {
                        total: Some(17163091968),
                        used: Some(668274688),
                    },
                    throttle_info: None,
                };

                data.update_with_timestamp(&stats, 1.0, datetime.and_utc().timestamp_millis());
            }
        }

        Arc::new(RwLock::new(data))
    }
}
