mod apply_revealer;
mod graphs_window;
mod header;
mod info_row;
mod page_section;
mod root_stack;

use std::time::Duration;

#[cfg(feature = "bench")]
pub use graphs_window::plot::{Plot, PlotData};

use crate::{APP_ID, GUI_VERSION};
use anyhow::{anyhow, Context};
use apply_revealer::{ApplyRevealer, ApplyRevealerMsg};
use graphs_window::GraphsWindow;
use gtk::{
    glib::{self, clone},
    prelude::{BoxExt, DialogExtManual, GtkWindowExt, OrientableExt},
    ApplicationWindow, ButtonsType, MessageDialog, MessageType,
};
use header::Header;
use lact_client::DaemonClient;
use lact_schema::{DeviceStats, GIT_COMMIT};
use relm4::{tokio, Component, ComponentController, ComponentParts, ComponentSender};
use root_stack::RootStack;
use tracing::{debug, error, trace, warn};

const STATS_POLL_INTERVAL_MS: u64 = 250;

pub struct AppModel {
    daemon_client: DaemonClient,
    graphs_window: GraphsWindow,
    root_stack: RootStack,
    header: relm4::Controller<Header>,
    apply_revealer: relm4::Controller<ApplyRevealer>,
    stats_task_handle: Option<glib::JoinHandle<()>>,
}

#[relm4::component(pub)]
impl Component for AppModel {
    type Init = (DaemonClient, Option<anyhow::Error>);

    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[name = "root_window"]
        gtk::ApplicationWindow {
            set_title: Some("LACT"),
            set_default_width: 600,
            set_default_height: 860,
            set_icon_name: Some(APP_ID),
            set_titlebar: Some(model.header.widget()),

            #[name = "root_box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                model.root_stack.container.clone(),
                model.apply_revealer.widget(),
            }
        }
    }

    fn init(
        (daemon_client, conn_err): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let system_info_buf = daemon_client
            .get_system_info()
            .expect("Could not fetch system info");
        let system_info = system_info_buf.inner().expect("Invalid system info buffer");

        let devices_buf = daemon_client
            .list_devices()
            .expect("Could not list devices");
        let devices = devices_buf.inner().expect("Could not access devices");

        if system_info.version != GUI_VERSION || system_info.commit != Some(GIT_COMMIT) {
            let err = anyhow!("Version mismatch between GUI and daemon ({GUI_VERSION}-{GIT_COMMIT} vs {}-{})! Make sure you have restarted the service if you have updated LACT.", system_info.version, system_info.commit.unwrap_or_default());
            sender.input(AppMsg::Error(err));
        }

        let root_stack = RootStack::new(system_info, daemon_client.embedded);

        let header = Header::builder()
            .launch((devices, root_stack.container.clone()))
            .forward(sender.input_sender(), |msg| msg);

        let apply_revealer = ApplyRevealer::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| msg);

        let graphs_window = GraphsWindow::new();

        let model = AppModel {
            daemon_client,
            graphs_window,
            root_stack,
            apply_revealer,
            header,
            stats_task_handle: None,
        };

        let widgets = view_output!();

        if let Some(err) = conn_err {
            sender.input(AppMsg::Error(err));
        }

        sender.input(AppMsg::GpuChanged(0));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        trace!("update {msg:#?}");
        match msg {
            AppMsg::Error(err) => {
                show_error(root, err);
            }
            AppMsg::GpuChanged(_) => match self.current_gpu_id() {
                Some(new_gpu_id) => {
                    if let Err(err) = self.update_gpu_data(new_gpu_id, sender.clone()) {
                        show_error(root, err);
                    }
                }
                None => show_error(root, anyhow!("No GPUs detected")),
            },
            AppMsg::Stats(stats) => {
                self.root_stack.info_page.set_stats(&stats);
                self.root_stack.thermals_page.set_stats(&stats, false);
                self.root_stack.oc_page.set_stats(&stats, false);
                self.graphs_window.set_stats(&stats);
            }
            AppMsg::ApplyChanges => todo!(),
            AppMsg::RevertChanges => {
                if let Some(gpu_id) = self.current_gpu_id() {
                    if let Err(err) = self.update_gpu_data(gpu_id, sender.clone()) {
                        show_error(root, err);
                    }
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl AppModel {
    fn current_gpu_id(&self) -> Option<String> {
        self.header.model().selected_gpu_id()
    }

    fn update_gpu_data(
        &mut self,
        gpu_id: String,
        sender: ComponentSender<AppModel>,
    ) -> anyhow::Result<()> {
        if let Some(stats_task) = self.stats_task_handle.take() {
            stats_task.abort();
        }

        debug!("setting initial info for gpu {gpu_id}");

        let info_buf = self
            .daemon_client
            .get_device_info(&gpu_id)
            .context("Could not fetch info")?;
        let info = info_buf.inner()?;

        let stats = self
            .daemon_client
            .get_device_stats(&gpu_id)
            .context("Could not fetch stats")?
            .inner()?;

        self.root_stack.info_page.set_info(&info);
        self.root_stack.oc_page.set_info(&info);

        let vram_clock_ratio = info
            .drm_info
            .as_ref()
            .map(|info| info.vram_clock_ratio)
            .unwrap_or(1.0);
        self.graphs_window.set_vram_clock_ratio(vram_clock_ratio);

        self.root_stack.oc_page.set_stats(&stats, true);
        self.root_stack.thermals_page.set_stats(&stats, true);
        self.root_stack.info_page.set_stats(&stats);

        let maybe_clocks_table = match self.daemon_client.get_device_clocks_info(&gpu_id) {
            Ok(clocks_buf) => match clocks_buf.inner() {
                Ok(info) => info.table,
                Err(err) => {
                    debug!("could not extract clocks info: {err:?}");
                    None
                }
            },
            Err(err) => {
                debug!("could not fetch clocks info: {err:?}");
                None
            }
        };
        self.root_stack.oc_page.set_clocks_table(maybe_clocks_table);

        let maybe_modes_table = match self.daemon_client.get_device_power_profile_modes(&gpu_id) {
            Ok(buf) => match buf.inner() {
                Ok(table) => Some(table),
                Err(err) => {
                    debug!("Could not extract profile modes table: {err:?}");
                    None
                }
            },
            Err(err) => {
                debug!("Could not get profile modes table: {err:?}");
                None
            }
        };
        self.root_stack
            .oc_page
            .performance_frame
            .set_power_profile_modes(maybe_modes_table);

        match self
            .daemon_client
            .get_power_states(&gpu_id)
            .and_then(|states| states.inner())
        {
            Ok(power_states) => {
                self.root_stack
                    .oc_page
                    .power_states_frame
                    .set_power_states(power_states);
            }
            Err(err) => warn!("could not get power states: {err:?}"),
        }

        self.root_stack.thermals_page.set_info(&info);

        // Show apply button on setting changes
        // This is done here because new widgets may appear after applying settings (like fan curve points) which should be connected
        let show_revealer = clone!(
            #[strong(rename_to = apply_sender)]
            self.apply_revealer.sender(),
            move || {
                apply_sender.send(ApplyRevealerMsg::Show).unwrap();
            }
        );

        self.root_stack
            .thermals_page
            .connect_settings_changed(show_revealer.clone());

        self.root_stack
            .oc_page
            .connect_settings_changed(show_revealer);

        self.apply_revealer
            .sender()
            .send(ApplyRevealerMsg::Hide)
            .unwrap();

        self.graphs_window.clear();

        self.stats_task_handle = Some(start_stats_update_loop(
            gpu_id.to_owned(),
            self.daemon_client.clone(),
            sender,
        ));

        Ok(())
    }
}

#[derive(Debug)]
pub enum AppMsg {
    Error(anyhow::Error),
    GpuChanged(usize),
    Stats(DeviceStats),
    ApplyChanges,
    RevertChanges,
}

fn show_error(parent: &ApplicationWindow, err: anyhow::Error) {
    let text = format!("{err:?}")
        .lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join("\n");
    warn!("{text}");
    let diag = MessageDialog::builder()
        .title("Error")
        .message_type(MessageType::Error)
        .text(text)
        .buttons(ButtonsType::Close)
        .transient_for(parent)
        .build();
    diag.run_async(|diag, _| {
        diag.close();
    })
}

fn start_stats_update_loop(
    gpu_id: String,
    daemon_client: DaemonClient,
    sender: ComponentSender<AppModel>,
) -> glib::JoinHandle<()> {
    debug!("spawning new stats update task with {STATS_POLL_INTERVAL_MS}ms interval");
    let duration = Duration::from_millis(STATS_POLL_INTERVAL_MS);
    relm4::spawn_local(async move {
        loop {
            tokio::time::sleep(duration).await;

            match daemon_client
                .get_device_stats(&gpu_id)
                .and_then(|buffer| buffer.inner())
            {
                Ok(stats) => {
                    sender.input(AppMsg::Stats(stats));
                }
                Err(err) => {
                    error!("could not fetch stats: {err:#}");
                }
            }
        }
    })
}
