use crate::app::info_row::InfoRow;
use gtk::glib::{self, Object};

glib::wrapper! {
    pub struct InfoRowLevel(ObjectSubclass<imp::InfoRowLevel>)
        @extends InfoRow, gtk::Box, gtk::Widget,
        @implements gtk::Orientable, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl InfoRowLevel {
    pub fn new(name: &str, value: &str, level: f64) -> Self {
        Object::builder()
            .property("name", name)
            .property("value", value)
            .property("level-value", level)
            .build()
    }
}

impl Default for InfoRowLevel {
    fn default() -> Self {
        Object::builder().build()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::{ControlFlow, Properties};
    use gtk::{LevelBar, TickCallbackId, glib, prelude::*, subclass::prelude::*};
    use relm4::view;

    use crate::app::info_row::{InfoRow, InfoRowExt};

    struct AnimationState {
        displayed_value: f64,
        velocity: f64,
        tick_id: Option<TickCallbackId>,
        last_frame_time: Option<std::time::Instant>,
    }

    impl Default for AnimationState {
        fn default() -> Self {
            Self {
                displayed_value: 0.0,
                velocity: 0.0,
                tick_id: None,
                last_frame_time: None,
            }
        }
    }

    const STIFFNESS: f64 = 200.0;
    const DAMPING_RATIO: f64 = 1.0;
    const MASS: f64 = 0.5;
    const EPSILON: f64 = 0.00025;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::InfoRowLevel)]
    pub struct InfoRowLevel {
        #[property(get, set = Self::set_level_value)]
        level_value: RefCell<f64>,
        animation: RefCell<AnimationState>,
        level_bar: RefCell<Option<LevelBar>>,
    }

    impl InfoRowLevel {
        fn set_level_value(&self, value: f64) {
            let old = *self.level_value.borrow();
            if (old - value).abs() < f64::EPSILON {
                return;
            }

            self.level_value.replace(value);
            self.start_animation(value);
        }

        /// Starts a spring-physics animation
        /// https://gitlab.gnome.org/GNOME/libadwaita/-/blob/main/src/adw-spring-animation.c
        fn start_animation(&self, _target: f64) {
            let obj = self.obj().to_owned();
            let level_bar = self.level_bar.borrow().clone();

            let Some(level_bar) = level_bar else {
                return;
            };

            let mut animation = self.animation.borrow_mut();

            if animation.tick_id.is_none() {
                let obj_weak = glib::WeakRef::new();
                obj_weak.set(Some(&obj));
                let level_bar_weak = glib::WeakRef::new();
                level_bar_weak.set(Some(&level_bar));

                let id = level_bar.add_tick_callback(move |_widget, _frame_clock| {
                    let Some(obj) = obj_weak.upgrade() else {
                        return ControlFlow::Break;
                    };
                    let Some(level_bar) = level_bar_weak.upgrade() else {
                        return ControlFlow::Break;
                    };

                    let imp = obj.imp();
                    let target = *imp.level_value.borrow();
                    let mut anim = imp.animation.borrow_mut();

                    let now = std::time::Instant::now();
                    let dt = anim
                        .last_frame_time
                        .map(|t| (now - t).as_secs_f64())
                        .unwrap_or(0.016);
                    anim.last_frame_time = Some(now);

                    let damping = 2.0 * DAMPING_RATIO * (STIFFNESS * MASS).sqrt();
                    let spring_force = STIFFNESS * (target - anim.displayed_value);
                    let damping_force = damping * anim.velocity;
                    let acceleration = (spring_force - damping_force) / MASS;

                    anim.velocity += acceleration * dt;
                    anim.displayed_value += anim.velocity * dt;

                    let settled = anim.velocity.abs() < EPSILON
                        && (target - anim.displayed_value).abs() < EPSILON;

                    if settled {
                        anim.displayed_value = target;
                        anim.velocity = 0.0;
                        anim.tick_id = None;
                        anim.last_frame_time = None;
                    }

                    level_bar.set_value(anim.displayed_value);

                    if settled {
                        ControlFlow::Break
                    } else {
                        ControlFlow::Continue
                    }
                });
                animation.tick_id = Some(id);
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InfoRowLevel {
        const NAME: &'static str = "InfoRowLevel";
        type Type = super::InfoRowLevel;
        type ParentType = InfoRow;
    }

    #[glib::derived_properties]
    impl ObjectImpl for InfoRowLevel {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            view! {
                #[local_ref]
                obj {
                    #[name(level_bar)]
                    append_child = &LevelBar {
                        set_hexpand: true,
                        set_orientation: gtk::Orientation::Horizontal,
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_LOW),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_HIGH),
                        remove_offset_value: Some(gtk::LEVEL_BAR_OFFSET_FULL),
                    }
                }
            }

            self.level_bar.replace(Some(level_bar));
        }
    }

    impl WidgetImpl for InfoRowLevel {}
    impl BoxImpl for InfoRowLevel {}
}
