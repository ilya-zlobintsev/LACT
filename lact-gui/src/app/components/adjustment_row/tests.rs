use super::*;
use relm4::{Component, ComponentController};
use std::{cell::Cell, rc::Rc};

fn drain_events() {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
}

fn launch_row() -> (relm4::Controller<AdjustmentRow>, Rc<Cell<usize>>) {
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let row = AdjustmentRow::builder()
        .launch(AdjustmentRowInit {
            title: "Adjustment".into(),
            value: 50.0,
            lower: 0.0,
            upper: 100.0,
            ..Default::default()
        })
        .connect_receiver(move |_, ()| changes_clone.set(changes_clone.get() + 1));
    drain_events();
    assert_eq!(changes.get(), 0);
    assert_eq!(row.model().get_changed_value(), None);
    (row, changes)
}

#[test]
fn refresh_is_silent_and_edits_notify() {
    gtk::test_synced(|| {
        let (row, changes) = launch_row();
        // A disjoint range must update atomically, without clamping to the old bounds.
        row.emit(AdjustmentRowMsg::Configure {
            value: 300.0,
            lower: 200.0,
            upper: 400.0,
        });
        drain_events();
        assert_eq!(row.model().get_value(), 300.0);
        assert_eq!(row.model().get_changed_value(), None);
        assert_eq!(changes.get(), 0);

        row.widgets().scale.set_value(350.0);
        drain_events();
        assert_eq!(row.model().get_changed_value(), Some(350.0));
        assert!(changes.replace(0) > 0);

        row.emit(AdjustmentRowMsg::Configure {
            value: 250.0,
            lower: 200.0,
            upper: 400.0,
        });
        drain_events();
        assert_eq!(row.model().get_changed_value(), None);
        assert_eq!(changes.get(), 0);

        // Reset is a user action, even though it arrives as a component message.
        row.emit(AdjustmentRowMsg::SetValue(300.0));
        drain_events();
        assert_eq!(row.model().get_changed_value(), Some(300.0));
        assert!(changes.get() > 0);
    });
}

#[test]
fn display_ratio_preserves_values_and_pending_edits() {
    gtk::test_synced(|| {
        let (row, changes) = launch_row();
        row.emit(AdjustmentRowMsg::ValueRatio(8.0));
        drain_events();
        assert_eq!(row.widgets().spinbutton.value(), 400.0);
        assert_eq!(row.model().get_value(), 50.0);
        assert_eq!(row.model().get_changed_value(), None);
        assert_eq!(changes.get(), 0);

        row.widgets().scale.set_value(480.0);
        drain_events();
        assert_eq!(row.model().get_changed_value(), Some(60.0));
        assert!(changes.replace(0) > 0);

        row.emit(AdjustmentRowMsg::ValueRatio(1.0));
        drain_events();
        assert_eq!(row.widgets().spinbutton.value(), 60.0);
        assert_eq!(row.model().get_changed_value(), Some(60.0));
        assert_eq!(changes.get(), 0);

        row.emit(AdjustmentRowMsg::ValueRatio(8.0));
        row.emit(AdjustmentRowMsg::Configure {
            value: -10.0,
            lower: -100.0,
            upper: 100.0,
        });
        drain_events();
        assert_eq!(row.widgets().spinbutton.value(), -80.0);
        assert_eq!(row.model().get_value(), -10.0);
        assert_eq!(row.model().get_changed_value(), None);
        assert_eq!(changes.get(), 0);

        row.emit(AdjustmentRowMsg::SetValue(0.0));
        drain_events();
        assert_eq!(row.model().get_changed_value(), Some(0.0));
        assert!(changes.get() > 0);
    });
}

#[test]
fn spinbutton_text_notifies_before_commit() {
    gtk::test_synced(|| {
        let (row, changes) = launch_row();
        row.widgets().spinbutton.set_text("75");
        drain_events();
        assert!(changes.replace(0) > 0);
        assert_eq!(row.model().get_value(), 50.0);

        row.widgets().spinbutton.update();
        drain_events();
        assert_eq!(row.model().get_changed_value(), Some(75.0));
        assert!(changes.get() > 0);
    });
}
