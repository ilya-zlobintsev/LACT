use super::*;

#[test]
#[ignore = "requires a GTK display; run explicitly with --ignored"]
fn experimental_power_mode_is_explicit_and_restores_native_bounds() {
    adw::init().unwrap();
    let context = gtk::glib::MainContext::default();
    let _guard = context.acquire().unwrap();
    let drain = || {
        while context.pending() {
            context.iteration(false);
        }
    };
    let frame = PowerFrame::detach_default();
    let cap_min = || frame.model().cap_min(frame.model().power.cap_min);
    let native_stats = PowerStats {
        cap_current: Some(300.0),
        cap_min: Some(250.0),
        cap_max: Some(325.0),
        cap_default: Some(300.0),
        ..Default::default()
    };
    frame.emit(PowerFrameMsg::NvidiaMode(Some(NvidiaPowerCapMode::Nvml)));
    frame.emit(PowerFrameMsg::PowerStats(native_stats.clone()));
    drain();
    assert!(frame.widgets().experimental_controls.is_visible());
    assert!(!frame.widgets().ioctl_toggle.is_active());
    assert_eq!(cap_min(), 250.0);
    assert_eq!(frame.model().get_user_cap(), None);

    // The factory recreates the power limit row; it must stay above the toggle.
    let list = frame.model().power_row.widget().clone();
    let toggle = (0..)
        .map_while(|index| list.row_at_index(index))
        .position(|row| row == frame.widgets().experimental_controls)
        .unwrap();
    assert!((0..toggle).any(|index| list.row_at_index(index as i32).unwrap().is_visible()));

    // A user opt-in extends the range; opt-out clamps the pending edit and
    // exposes it to AppModel so an old sub-minimum config is not resubmitted.
    frame.widgets().ioctl_toggle.set_active(true);
    drain();
    assert_eq!(cap_min(), 30.0);
    frame
        .model()
        .power_row
        .send(&(), AdjustmentRowMsg::SetValue(150.0));
    drain();
    assert_eq!(frame.model().get_user_cap(), Some(150.0));
    frame.widgets().ioctl_toggle.set_active(false);
    drain();
    assert_eq!(cap_min(), 250.0);
    assert_eq!(frame.model().get_user_cap(), Some(250.0));

    // Restore an opted-in profile and its lower cap together, without
    // treating programmatic checkbox updates as user mode changes.
    frame.emit(PowerFrameMsg::NvidiaMode(Some(NvidiaPowerCapMode::Ioctl)));
    frame.emit(PowerFrameMsg::PowerStats(PowerStats {
        cap_current: Some(150.0),
        ..native_stats.clone()
    }));
    drain();
    assert!(frame.widgets().ioctl_toggle.is_active());
    assert_eq!(frame.model().power_row.get(&()).unwrap().get_value(), 150.0);
    assert_eq!(frame.model().get_user_cap(), None);
    frame.emit(PowerFrameMsg::Reset);
    drain();
    assert_eq!(frame.model().get_user_cap(), Some(300.0));
    assert_eq!(
        frame.model().nvidia_power_cap_mode(),
        Some(NvidiaPowerCapMode::Ioctl)
    );

    // Other drivers never see the toggle or report a mode.
    frame.emit(PowerFrameMsg::NvidiaMode(None));
    frame.emit(PowerFrameMsg::PowerStats(native_stats));
    drain();
    assert!(!frame.widgets().experimental_controls.is_visible());
    assert_eq!(frame.model().nvidia_power_cap_mode(), None);
    assert_eq!(cap_min(), 250.0);
}
