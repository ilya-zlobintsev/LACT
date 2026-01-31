use super::cubic_spline::cubic_spline_interpolation;
use super::to_texture_ext::ToTextureExt;
use super::PlotColorScheme;
use crate::app::formatting;
use crate::app::graphs_window::stat::{StatType, StatsData};
use anyhow::Context;
use cairo::{Context as CairoContext, ImageSurface};
use gtk::gdk::MemoryTexture;
use indexmap::IndexSet;
use plotters::prelude::*;
use plotters::style::text_anchor::Pos;
use plotters_cairo::CairoBackend;
use relm4::tokio::sync::broadcast;
use std::cmp::{self, max};
use std::fmt::Write;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, RwLock};
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use tracing::error;

enum Request {
    Terminate,
    Render(RenderRequest),
}

pub struct RenderRequest {
    pub title: String,

    pub print_extra_info: bool,
    pub data: Arc<RwLock<StatsData>>,
    pub stats: Vec<StatType>,
    pub colors: PlotColorScheme,

    pub width: u32,
    pub height: u32,
    pub scale_factor: i32,

    pub time_period_seconds: i64,
}

#[derive(Default)]
struct RenderThreadState {
    request_condition_variable: std::sync::Condvar,
    last_texture: Mutex<Option<MemoryTexture>>,
    current_request: Mutex<Option<Request>>,
}

/// A rendering thread that will listen for rendering requests and process them asynchronously.
/// Requests that weren't processed in time or resulted in error are dropped.
pub struct RenderThread {
    /// Shared state is between the main thread and the rendering thread.
    state: Arc<RenderThreadState>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    render_notifier: broadcast::Sender<()>,
}

/// Ensure the rendering thread is terminated properly when the RenderThread object is dropped.
/// We send Request::Terminate to swiftly terminate rendering thread and then join to let it finish last render.
impl Drop for RenderThread {
    fn drop(&mut self) {
        self.state
            .current_request
            .lock()
            .unwrap()
            .replace(Request::Terminate);
        self.state.request_condition_variable.notify_all();

        self.thread_handle.take().map(|handle| handle.join().ok());
    }
}

impl RenderThread {
    pub fn new() -> Self {
        let state = Arc::new(RenderThreadState::default());
        let render_tx = broadcast::Sender::new(1);

        let thread_handle = std::thread::Builder::new()
            .name("Plot-Renderer".to_owned())
            // Render thread is very unimportant, skipping frames and rendering slowly is ok
            .spawn_with_priority(ThreadPriority::Min, {
                let state = state.clone();
                let render_tx = render_tx.clone();
                move |_| loop {
                    let RenderThreadState {
                        request_condition_variable,
                        last_texture,
                        current_request,
                    } = &*state;

                    // Wait until there is a new request (blocking if there is none).
                    let mut current_request = request_condition_variable
                        .wait_while(current_request.lock().unwrap(), |pending_request| {
                            pending_request.is_none()
                        })
                        .unwrap();

                    match current_request.take() {
                        Some(Request::Render(render_request)) => {
                            process_request(render_request, last_texture);
                        }
                        // Terminate the thread if a Terminate request is received.
                        Some(Request::Terminate) => break,
                        None => {}
                    }
                    let _ = render_tx.send(());
                }
            })
            .unwrap();

        Self {
            state,
            thread_handle: Some(thread_handle),
            render_notifier: render_tx,
        }
    }

    pub fn render_notifier(&self) -> broadcast::Receiver<()> {
        self.render_notifier.subscribe()
    }

    /// Replace the current render request with a new one (effectively dropping possible pending frame)
    /// Returns dropped request if any
    pub fn replace_render_request(&self, request: RenderRequest) -> Option<RenderRequest> {
        let mut current_request = self.state.current_request.lock().unwrap();
        let result = current_request.replace(Request::Render(request));
        self.state.request_condition_variable.notify_one(); // Notify the thread to start rendering.

        match result? {
            Request::Render(render) => Some(render),
            Request::Terminate => None,
        }
    }

    /// Return the last texture.
    /// Requests that weren't processed in time or resulted in error are dropped.
    pub fn get_last_texture(&self) -> Option<MemoryTexture> {
        self.state.last_texture.lock().unwrap().deref().clone()
    }
}

pub(super) fn process_request(
    render_request: RenderRequest,
    last_texture: &Mutex<Option<MemoryTexture>>,
) {
    // Create a new ImageSurface for Cairo rendering.
    let mut surface = ImageSurface::create(
        cairo::Format::ARgb32,
        (render_request.width) as i32 * render_request.scale_factor,
        render_request.height as i32 * render_request.scale_factor,
    )
    .unwrap();

    surface.set_device_scale(
        render_request.scale_factor as f64,
        render_request.scale_factor as f64,
    );

    let cairo_context = CairoContext::new(&surface).unwrap();

    let cairo_backend = CairoBackend::new(
        &cairo_context,
        (render_request.width, render_request.height),
    )
    .unwrap();

    if let Err(err) = render_request.draw(cairo_backend) {
        error!("Failed to plot chart: {err:?}")
    }

    match (
        surface.to_texture(),
        last_texture.lock().unwrap().deref_mut(),
    ) {
        // Successfully generated a new texture, but the old texture is also there
        (Some(texture), Some(last_texture)) => {
            *last_texture = texture;
        }
        // If texture conversion failed, keep the old texture if it's present.
        (None, None) => {
            error!("Failed to convert cairo surface to gdk texture, not overwriting old one");
        }
        // Update the last texture, if The old texture wasn't ever generated (None),
        // No matter the result of conversion
        (result, last_texture) => {
            *last_texture = result;
        }
    };
}

// Implement the default constructor for RenderThread using the `new` method.
impl Default for RenderThread {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderRequest {
    // Method to handle the actual drawing of the chart.
    pub fn draw<'a, DB>(&self, backend: DB) -> anyhow::Result<()>
    where
        DB: DrawingBackend + 'a,
        <DB as plotters::prelude::DrawingBackend>::ErrorType: 'static,
    {
        let root = backend.into_drawing_area(); // Create the drawing area.

        let data_guard = self.data.read().unwrap();

        // Determine the start and end dates of the data series.
        let start_date = data_guard.first_timestamp().unwrap_or_default();
        let end_date = data_guard.last_timestamp().unwrap_or_default();

        let data = data_guard.get_stats(&self.stats).collect::<Vec<_>>();

        let value_suffix = if self.stats.len() >= 2 {
            let mut metric = self.stats[0].metric();
            // Only display a suffix if it's the same across all metrics on the plot
            for stat in &self.stats[1..] {
                if stat.metric() != metric {
                    metric = "";
                    break;
                }
            }
            metric
        } else {
            self.stats
                .first()
                .map(|stat| stat.metric())
                .unwrap_or_default()
        };

        // Calculate the maximum value for the y-axis.
        let mut maximum_value = data
            .iter()
            .flat_map(|(_, list)| list.iter().map(|(_, value)| *value))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(cmp::Ordering::Equal))
            .unwrap_or_default();

        if maximum_value < 100.0 {
            maximum_value = 100.0;
        }

        root.fill(&self.colors.background)?; // Fill the background with white color.

        let y_label_style = TextStyle {
            font: ("sans-serif", 18).into(),
            color: self.colors.text.to_backend_color(),
            pos: Pos::default(),
        };
        let y_label_area_size = root
            .estimate_text_size(&format!("{maximum_value}{value_suffix}"), &y_label_style)?
            .0
            + 10;

        // Set up the main chart with axes and labels.
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(25)
            .y_label_area_size(y_label_area_size)
            .margin(10)
            .margin_top(20)
            .caption(self.title.as_str(), ("sans-serif", 24, &self.colors.text))
            .build_cartesian_2d(
                start_date..max(end_date, start_date + self.time_period_seconds * 1000),
                0f64..maximum_value,
            )?;

        // Configure the x-axis and y-axis mesh.
        chart
            .configure_mesh()
            .axis_style(self.colors.border_secondary)
            .bold_line_style(self.colors.border)
            .x_label_formatter(&formatting::fmt_timestamp_to_dt)
            .y_label_formatter(&|x| format!("{x}{value_suffix}"))
            .x_labels(5)
            .y_labels(10)
            .label_style(y_label_style.clone())
            .draw()
            .context("Failed to draw mesh")?;

        // Draw the main line series using cubic spline interpolation.
        for (idx, (stat_type, data)) in data.iter().enumerate() {
            let current_value = data.last().map(|(_, val)| *val).unwrap_or(0.0);
            let max_value = data
                .iter()
                .map(|(_, val)| *val)
                .reduce(f64::max)
                .unwrap_or(0.0);
            let avg_value = data.iter().map(|(_, val)| *val).sum::<f64>() / data.len() as f64;

            let stat_suffix = stat_type.metric();

            let mut stat_label =
                format!("{}: {current_value:.1}{stat_suffix}", stat_type.display(),);
            if self.print_extra_info {
                if stat_type.show_peak() {
                    write!(stat_label, ", Peak {max_value:.1}{stat_suffix}").unwrap();
                }

                write!(stat_label, ", Avg {avg_value:.1}{stat_suffix}").unwrap();
            }

            // Evaluate fixed number of points per spline segment
            const POINTS_PER_SEGMENT: usize = 4;

            chart
                .draw_series(LineSeries::new(
                    cubic_spline_interpolation(data).flat_map(
                        move |((first_time, second_time), segment)| {
                            let segment_duration = (second_time - first_time) as f64;
                            let step = segment_duration / (POINTS_PER_SEGMENT as f64 - 1.0);

                            // Evaluate spline at POINTS_PER_SEGMENT evenly spaced points per segment.
                            (0..POINTS_PER_SEGMENT).map(move |i| {
                                let offset = (i as f64 * step).round() as i64;
                                let current_date = first_time + offset;
                                let current_date = current_date.min(second_time - 1);
                                (current_date, segment.evaluate(current_date))
                            })
                        },
                    ),
                    Palette99::pick(idx).stroke_width(2),
                ))
                .context("Failed to draw series")?
                .label(stat_label)
                .legend(move |(x, y)| {
                    let offset = 7;
                    Rectangle::new(
                        [(x - offset, y - offset), (x + offset, y + offset)],
                        Palette99::pick(idx).filled(),
                    )
                });
        }

        // Draw throttling series
        if data_guard
            .throttling_sections()
            .iter()
            .any(|section| !section.is_empty())
        {
            let throttling_color = self.colors.throttling.mix(0.5).filled();
            let names: IndexSet<&str> = data_guard
                .throttling_sections()
                .iter()
                .flat_map(|section| section.iter().flat_map(|(_, values)| values))
                .map(|value| value.as_str())
                .collect();

            chart
                .draw_series(
                    data_guard
                        .throttling_sections()
                        .iter()
                        .filter_map(|section| {
                            let first = section.first()?;
                            Some((
                                first.0,
                                section.last().map(|(ts, _)| *ts).unwrap_or(first.0),
                            ))
                        })
                        .map(|(start, end)| {
                            Rectangle::new([(start, 0f64), (end, maximum_value)], throttling_color)
                        }),
                )
                .context("Failed to draw throttling histogram")?
                .label(format!(
                    "Throttling: {}",
                    names.into_iter().collect::<Vec<_>>().join(",")
                ))
                .legend(move |(x, y)| {
                    let offset = 7;
                    Rectangle::new(
                        [(x - offset, y - offset), (x + offset, y + offset)],
                        throttling_color,
                    )
                });
        }

        // Configure and draw series labels (the legend).
        chart
            .configure_series_labels()
            .margin(20)
            .label_font(("sans-serif", 14, &self.colors.text))
            .position(SeriesLabelPosition::UpperLeft)
            .background_style(self.colors.background.mix(0.6))
            .draw()
            .context("Failed to draw series labels")?;

        root.present()?; // Present the final image.
        Ok(())
    }
}
