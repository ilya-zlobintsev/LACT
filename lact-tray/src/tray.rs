//! StatusNotifierItem implementation of the tray icon.

use std::sync::{Arc, Mutex};

use anyhow::Context as _;
use i18n_embed_fl::fl;
use ksni::{
    ToolTip, TrayMethods as _,
    menu::{MenuItem, StandardItem},
};
use lact_client::DaemonClient;
use tokio::sync::mpsc;
use tracing::debug;

use super::{
    APP_ID, I18N,
    snapshot_ext::{Snapshot, SnapshotExt},
};

/// Something the user did in the tray, sent to the caller which owns the reaction to it.
pub enum TrayEvent {
    /// The host is about to open the menu, so the values it shows are worth refreshing.
    Show,
    /// The user asked to open the GUI.
    ShowWindow,
    /// The user asked to quit from the tray menu.
    Quit,
}

/// Tray icon: registers the StatusNotifierItem and renders the values it is given.
///
/// It neither fetches nor decides anything: it reports [`TrayEvent`]s and the caller publishes
/// values with [`TrayHandle::update`]. The `ksni::Tray` callbacks live on the private
/// [`LactTrayImpl`].
pub struct LactTray(LactTrayImpl);

impl LactTray {
    /// Creates the tray, reporting everything the user does on `events`.
    ///
    /// The channel is unbounded because the events are sent from the library's callbacks, which
    /// cannot await: `send` must never drop an event (a lost `Quit` would leave the tray running).
    pub fn new(events: mpsc::UnboundedSender<TrayEvent>) -> Self {
        Self(LactTrayImpl::new(events))
    }

    /// Registers the StatusNotifierItem and returns the handle used to publish new values.
    pub async fn spawn(&self) -> anyhow::Result<TrayHandle> {
        // `ksni` takes the service by value, hence the clone - all of its state is shared.
        let handle = self
            .0
            .clone()
            .spawn()
            .await
            .context("Could not register the tray icon")?;

        Ok(TrayHandle {
            snapshot: self.0.snapshot.clone(),
            handle,
        })
    }
}

/// Handle to the registered StatusNotifierItem, used to publish freshly fetched values.
///
/// Hides the service type, which is private, and the closure that `ksni` requires for an update.
pub struct TrayHandle {
    snapshot: Arc<Mutex<Option<Snapshot>>>,
    handle: ksni::Handle<LactTrayImpl>,
}

impl TrayHandle {
    /// Reads the values of the selected GPU from the daemon and publishes them.
    pub async fn refresh(&self, daemon: &DaemonClient) {
        match daemon.snapshot().await {
            Ok(snapshot) => self.update(snapshot).await,
            Err(err) => debug!("could not read the GPU values: {err:#}"),
        }
    }

    /// Publishes the values and re-emits the StatusNotifierItem properties, so that a tooltip or
    /// menu which is currently open picks up the new values.
    pub async fn update(&self, snapshot: Snapshot) {
        *self
            .snapshot
            .lock()
            .expect("Tray snapshot mutex is poisoned") = Some(snapshot);

        // `ksni` only offers `update(|tray| ..)`: the closure is part of its signature even when
        // there is nothing to mutate, since the values are read through the property getters.
        let _ = self.handle.update(|_| {}).await;
    }
}

/// Reads of the item's values, as required by the `ksni::Tray` callbacks.
///
/// Cloned when the service is spawned, since `ksni` consumes it: every field is shared.
#[derive(Clone)]
struct LactTrayImpl {
    /// The last published values, used where fetching is not possible (e.g. the title).
    ///
    /// This is a `std` mutex and is never held across an `.await`.
    snapshot: Arc<Mutex<Option<Snapshot>>>,
    /// Reports what the user did, the caller decides what that means.
    events: mpsc::UnboundedSender<TrayEvent>,
}

impl LactTrayImpl {
    fn new(events: mpsc::UnboundedSender<TrayEvent>) -> Self {
        Self {
            snapshot: Arc::default(),
            events,
        }
    }

    /// The last published values, if any.
    fn snapshot(&self) -> Option<Snapshot> {
        self.snapshot
            .lock()
            .expect("Tray snapshot mutex is poisoned")
            .clone()
    }
}

impl ksni::Tray for LactTrayImpl {
    fn id(&self) -> String {
        APP_ID.to_owned()
    }

    fn title(&self) -> String {
        self.snapshot()
            .and_then(|snapshot| snapshot.gpu_name)
            .unwrap_or_else(|| "LACT".to_owned())
    }

    fn icon_name(&self) -> String {
        APP_ID.to_owned()
    }

    fn tool_tip(&self) -> ToolTip {
        // No event is sent here: the getter also runs when an update of our own re-reads the
        // properties, and asking for a fetch from it would loop forever. Values are polled on an
        // interval instead, so a read of the tooltip never needs to trigger anything.
        let snapshot = self.snapshot().unwrap_or_default();

        ToolTip {
            icon_name: APP_ID.to_owned(),
            title: snapshot
                .gpu_name
                .clone()
                .unwrap_or_else(|| "LACT".to_owned()),
            description: self.tooltip_description(),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.events.send(TrayEvent::ShowWindow);
    }

    fn menu_about_to_show(&mut self) {
        let _ = self.events.send(TrayEvent::Show);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let snapshot = self.snapshot().unwrap_or_default();
        let mut items = Vec::new();

        if let Some(gpu_name) = &snapshot.gpu_name {
            items.push(
                StandardItem {
                    label: gpu_name.clone(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }

        for (name, value) in self.rows() {
            items.push(
                StandardItem {
                    label: format!("{name}: {value}"),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }

        if !items.is_empty() {
            items.push(MenuItem::Separator);
        }

        items.push(
            StandardItem {
                label: fl!(I18N, "show-window"),
                icon_name: "window-new-symbolic".to_owned(),
                activate: Box::new(|tray: &mut LactTrayImpl| {
                    let _ = tray.events.send(TrayEvent::ShowWindow);
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(
            StandardItem {
                label: fl!(I18N, "quit"),
                icon_name: "application-exit-symbolic".to_owned(),
                // Menu callbacks are synchronous: the caller decides what quitting means.
                activate: Box::new(|tray: &mut LactTrayImpl| {
                    let _ = tray.events.send(TrayEvent::Quit);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

impl LactTrayImpl {
    /// Individual values in the order they are displayed.
    fn rows(&self) -> Vec<(String, String)> {
        let snapshot = self.snapshot().unwrap_or_default();

        let mut rows = Vec::new();

        if let Some(temperature) = snapshot.temperature {
            rows.push((fl!(I18N, "temperature"), format!("{temperature:.0}°C")));
        }
        if let Some(usage) = snapshot.usage_percent {
            rows.push((fl!(I18N, "gpu-usage"), format!("{usage}%")));
        }
        if let Some(power) = snapshot.power_watts {
            rows.push((fl!(I18N, "power-usage"), format!("{power:.1} W")));
        }

        rows
    }

    fn tooltip_description(&self) -> String {
        let rows = self.rows();
        if rows.is_empty() {
            fl!(I18N, "no-data")
        } else {
            rows.iter()
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}
