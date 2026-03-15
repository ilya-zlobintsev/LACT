use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use gtk::glib::{ControlFlow, WeakRef};
use gtk::{prelude::*, TickCallbackId};

const DURATION_MS: f64 = 200.0;

type UpdateCallback<W> = Option<Box<dyn Fn(&W, f64)>>;

pub(crate) struct LinearAnimation<W: IsA<gtk::Widget>> {
    inner: Rc<RefCell<Inner<W>>>,
}

struct Inner<W: IsA<gtk::Widget>> {
    tick_id: Option<TickCallbackId>,
    widget: WeakRef<W>,
    on_update: UpdateCallback<W>,
    start_value: f64,
    target: f64,
    start_time: Option<Instant>,
}

impl<W: IsA<gtk::Widget> + 'static> Default for LinearAnimation<W> {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                tick_id: None,
                widget: WeakRef::new(),
                on_update: None,
                start_value: 0.0,
                target: 0.0,
                start_time: None,
            })),
        }
    }
}

impl<W: IsA<gtk::Widget> + 'static> LinearAnimation<W> {
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
                start_value: 0.0,
                target: 0.0,
                start_time: None,
            })),
        }
    }

    pub fn animate_to(&self, target: f64) {
        let mut inner = self.inner.borrow_mut();

        if inner.tick_id.is_some() {
            inner.start_value = Self::interpolate(
                inner.start_value,
                inner.target,
                inner
                    .start_time
                    .map(|t| (Instant::now() - t).as_millis() as f64 / DURATION_MS)
                    .unwrap_or(0.0),
            );
        } else {
            inner.start_value = inner.target;
        }

        inner.target = target;
        inner.start_time = Some(Instant::now());

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

                let elapsed = inner
                    .start_time
                    .map(|t| (Instant::now() - t).as_millis() as f64)
                    .unwrap_or(0.0);

                let t = (elapsed / DURATION_MS).min(1.0);
                let value = Self::interpolate(inner.start_value, inner.target, t);

                if let Some(on_update) = inner.on_update.as_ref() {
                    on_update(&widget, value);
                }

                if t >= 1.0 {
                    inner.tick_id = None;
                    inner.start_time = None;
                    ControlFlow::Break
                } else {
                    ControlFlow::Continue
                }
            });

            inner.tick_id = Some(id);
        }
    }

    fn interpolate(start: f64, end: f64, t: f64) -> f64 {
        start + (end - start) * t
    }
}
