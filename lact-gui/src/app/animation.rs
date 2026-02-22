use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use gtk::glib::{ControlFlow, WeakRef};
use gtk::{TickCallbackId, prelude::*};

const STIFFNESS: f64 = 200.0;
const DAMPING_COEFFICIENT: f64 = 20.0;
const MASS: f64 = 0.5;
const EPSILON: f64 = 0.00025;

type UpdateCallback<W> = Option<Box<dyn Fn(&W, f64)>>;

/// Spring physics simulation (extracted for testability).
pub(crate) struct SpringPhysics {
    target: f64,
    displayed_value: f64,
    velocity: f64,
}

impl Default for SpringPhysics {
    fn default() -> Self {
        Self {
            target: 0.0,
            displayed_value: 0.0,
            velocity: 0.0,
        }
    }
}

impl SpringPhysics {
    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }

    pub fn displayed_value(&self) -> f64 {
        self.displayed_value
    }

    /// Advances physics by `dt` seconds. Returns `true` if settled.
    pub fn step(&mut self, dt: f64) -> bool {
        let spring_force = STIFFNESS * (self.target - self.displayed_value);
        let damping_force = DAMPING_COEFFICIENT * self.velocity;
        let acceleration = (spring_force - damping_force) / MASS;

        self.velocity += acceleration * dt;
        self.displayed_value += self.velocity * dt;

        let settled =
            self.velocity.abs() < EPSILON && (self.target - self.displayed_value).abs() < EPSILON;

        if settled {
            self.displayed_value = self.target;
            self.velocity = 0.0;
        }

        settled
    }
}

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
    on_update: UpdateCallback<W>,
    physics: SpringPhysics,
    last_frame_time: Option<Instant>,
}

impl<W: IsA<gtk::Widget> + 'static> Default for SpringAnimation<W> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                tick_id: None,
                widget: WeakRef::new(),
                on_update: None,
                physics: SpringPhysics::default(),
                last_frame_time: None,
            })),
        }
    }
}

impl<W: IsA<gtk::Widget> + 'static> SpringAnimation<W> {
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
                physics: SpringPhysics::default(),
                last_frame_time: None,
            })),
        }
    }

    /// Animate toward target. Updates target if already running.
    pub fn animate_to(&self, target: f64) {
        let mut inner = self.inner.borrow_mut();
        inner.physics.set_target(target);

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

                let settled = inner.physics.step(dt);

                if settled {
                    inner.tick_id = None;
                    inner.last_frame_time = None;
                }

                if let Some(on_update) = inner.on_update.as_ref() {
                    on_update(&widget, inner.physics.displayed_value());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converges_to_target() {
        // Verifies spring reaches target value within tolerance.
        let mut physics = SpringPhysics::default();
        physics.set_target(1.0);

        for _ in 0..50 {
            physics.step(0.016);
        }

        assert!((physics.displayed_value() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_critical_damping_no_overshoot() {
        // Critically damped spring should not overshoot target.
        let mut physics = SpringPhysics::default();
        physics.set_target(1.0);

        let mut max_value: f64 = 0.0;
        for _ in 0..100 {
            physics.step(0.016);
            max_value = max_value.max(physics.displayed_value());
        }

        assert!(max_value <= 1.0 + EPSILON);
    }

    #[test]
    fn test_zero_target() {
        // Target of zero should stay at zero.
        let mut physics = SpringPhysics::default();
        physics.set_target(0.0);

        for _ in 0..10 {
            physics.step(0.016);
        }

        assert!(physics.displayed_value().abs() < EPSILON);
    }

    #[test]
    fn test_negative_target() {
        // Spring handles negative target values correctly.
        let mut physics = SpringPhysics::default();
        physics.set_target(-0.5);

        for _ in 0..50 {
            physics.step(0.016);
        }

        assert!((physics.displayed_value() - (-0.5)).abs() < 0.01);
    }

    #[test]
    fn test_large_target() {
        // Spring handles large target values correctly.
        let mut physics = SpringPhysics::default();
        physics.set_target(100.0);

        for _ in 0..50 {
            physics.step(0.016);
        }

        assert!((physics.displayed_value() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_step_returns_settled() {
        // step() returns true when animation settles.
        let mut physics = SpringPhysics::default();
        physics.set_target(1.0);

        let mut settled = false;
        for _ in 0..100 {
            if physics.step(0.016) {
                settled = true;
                break;
            }
        }

        assert!(settled);
    }
}
