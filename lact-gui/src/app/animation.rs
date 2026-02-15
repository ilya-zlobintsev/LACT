use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use gtk::glib::{ControlFlow, WeakRef};
use gtk::{TickCallbackId, prelude::*};

const STIFFNESS: f64 = 200.0;
const DAMPING_COEFFICIENT: f64 = 20.0;
const MASS: f64 = 0.5;
const EPSILON: f64 = 0.00025;

/// Spring physics animation for widget transitions (replicates AdwSpringAnimation).
///
/// Memory: Rc<RefCell<Inner>> shared between struct and tick callback.
/// WeakRef ensures safe cleanup when widget destroyed.
pub struct SpringAnimation<W: IsA<gtk::Widget>> {
    inner: Rc<RefCell<Inner<W>>>,
}

struct Inner<W: IsA<gtk::Widget>> {
    tick_id: Option<TickCallbackId>,
    widget: WeakRef<W>,
    on_update: Option<Box<dyn Fn(&W, f64)>>,
    target: f64,
    displayed_value: f64,
    velocity: f64,
    last_frame_time: Option<Instant>,
}

impl<W: IsA<gtk::Widget> + 'static> Default for SpringAnimation<W> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                tick_id: None,
                widget: WeakRef::new(),
                on_update: None,
                target: 0.0,
                displayed_value: 0.0,
                velocity: 0.0,
                last_frame_time: None,
            })),
        }
    }
}

impl<W: IsA<gtk::Widget> + 'static> SpringAnimation<W> {
    /// Tick callback registered lazily on first animate_to().
    pub fn new<F>(widget: &W, on_update: F) -> Self
    where
        F: Fn(&W, f64) + 'static,
    {
        let widget_weak: WeakRef<W> = WeakRef::new();
        widget_weak.set(Some(widget));

        Self {
            inner: Rc::new(RefCell::new(Inner {
                tick_id: None,
                widget: widget_weak,
                on_update: Some(Box::new(on_update)),
                target: 0.0,
                displayed_value: 0.0,
                velocity: 0.0,
                last_frame_time: None,
            })),
        }
    }

    /// Animate toward target. Updates target if already running.
    pub fn animate_to(&self, target: f64) {
        let mut inner = self.inner.borrow_mut();
        inner.target = target;

        if inner.tick_id.is_none() {
            let inner_rc = Rc::clone(&self.inner);
            let Some(widget) = inner.widget.upgrade() else {
                return;
            };

            let id = widget.add_tick_callback(move |_w, _clock| {
                let mut inner = inner_rc.borrow_mut();

                let Some(widget) = inner.widget.upgrade() else {
                    inner.tick_id = None;
                    return ControlFlow::Break;
                };

                let now = Instant::now();
                let dt = inner
                    .last_frame_time
                    .map(|t| (now - t).as_secs_f64())
                    .unwrap_or(0.016);
                inner.last_frame_time = Some(now);

                // Spring physics: F = k*(target - pos) - c*velocity
                let spring_force = STIFFNESS * (inner.target - inner.displayed_value);
                let damping_force = DAMPING_COEFFICIENT * inner.velocity;
                let acceleration = (spring_force - damping_force) / MASS;

                inner.velocity += acceleration * dt;
                inner.displayed_value += inner.velocity * dt;

                let settled = inner.velocity.abs() < EPSILON
                    && (inner.target - inner.displayed_value).abs() < EPSILON;

                if settled {
                    inner.displayed_value = inner.target;
                    inner.velocity = 0.0;
                    inner.tick_id = None;
                    inner.last_frame_time = None;
                }

                if let Some(on_update) = inner.on_update.as_ref() {
                    on_update(&widget, inner.displayed_value);
                }

                if settled {
                    ControlFlow::Break
                } else {
                    ControlFlow::Continue
                }
            });

            inner.tick_id = Some(id);
        }
    }
}
