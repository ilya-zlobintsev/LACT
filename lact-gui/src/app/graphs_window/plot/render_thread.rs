use super::PlotColorScheme;
use super::cubic_spline::cubic_spline_interpolation;
use super::to_texture_ext::ToTextureExt;
use crate::app::graphs_window::stat::{StatType, StatsData};
use crate::app::utils::formatting;
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

#[allow(clippy::large_enum_variant)]
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
    /// Font size of the UI, in pixels.
    pub base_font_size: f64,

    pub time_period_seconds: i64,
}

/// A single entry of the plot legend, produced by the render thread and
/// displayed with GTK widgets.
#[derive(Debug, Clone)]
pub struct LegendLabel {
    pub text: String,
    pub color: RGBAColor,
}

#[derive(Default)]
struct RenderThreadState {
    request_condition_variable: std::sync::Condvar,
    last_texture: Mutex<Option<MemoryTexture>>,
    current_request: Mutex<Option<Request>>,
    labels: Mutex<Vec<LegendLabel>>,
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
                        labels,
                    } = &*state;

                    // Wait until there is a new request (blocking if there is none).
                    let mut current_request = request_condition_variable
                        .wait_while(current_request.lock().unwrap(), |pending_request| {
                            pending_request.is_none()
                        })
                        .unwrap();

                    match current_request.take() {
                        Some(Request::Render(render_request)) => {
                            process_request(render_request, last_texture, labels);
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

    /// Return the legend entries of the last rendered frame.
    pub fn get_labels(&self) -> Vec<LegendLabel> {
        self.state.labels.lock().unwrap().clone()
    }
}

pub(super) fn process_request(
    render_request: RenderRequest,
    last_texture: &Mutex<Option<MemoryTexture>>,
    labels: &Mutex<Vec<LegendLabel>>,
) {
    // Create a new ImageSurface for Cairo rendering.
    let mut surface = ImageSurface::create(
        cairo::Format::ARgb32,
        render_request.width as i32 * render_request.scale_factor,
        render_request.height as i32 * render_request.scale_factor,
    )
    .unwrap();

    surface.set_device_scale(
        render_request.scale_factor as f64,
        render_request.scale_factor as f64,
    );

    let cairo_context = CairoContext::new(&surface).unwrap();

    // The surface does not inherit the font options of GTK, without hinting the
    // small labels look blurry.
    if let Ok(mut font_options) = cairo::FontOptions::new() {
        font_options.set_antialias(cairo::Antialias::Gray);
        font_options.set_hint_style(cairo::HintStyle::Full);
        font_options.set_hint_metrics(cairo::HintMetrics::On);
        cairo_context.set_font_options(&font_options);
    }

    match render_request.draw(&cairo_context) {
        Ok(new_labels) => *labels.lock().unwrap() = new_labels,
        Err(err) => error!("Failed to plot chart: {err:?}"),
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

/// How much empty space to leave above the highest data point of a plot.
const Y_AXIS_HEADROOM: f64 = 1.1;

/// Round the value up to two significant digits.
fn nice_upper_bound(value: f64) -> f64 {
    let magnitude = 10f64.powf(value.log10().floor() - 1.0);
    (value / magnitude).ceil() * magnitude
}

/// Interpolate a series using a fixed number of points per spline segment.
fn interpolate_series(data: &[(i64, f64)]) -> Vec<(i64, f64)> {
    // Evaluate fixed number of points per spline segment
    const POINTS_PER_SEGMENT: usize = 4;

    cubic_spline_interpolation(data)
        .flat_map(move |((first_time, second_time), segment)| {
            let segment_duration = (second_time - first_time) as f64;
            let step = segment_duration / (POINTS_PER_SEGMENT as f64 - 1.0);

            // Evaluate spline at POINTS_PER_SEGMENT evenly spaced points per segment.
            (0..POINTS_PER_SEGMENT).map(move |i| {
                let offset = (i as f64 * step).round() as i64;
                let current_date = first_time + offset;
                let current_date = current_date.min(second_time - 1);
                (current_date, segment.evaluate(current_date))
            })
        })
        .collect()
}

struct LegendEntry {
    label: String,
    color: RGBAColor,
}

/// Format a legend label. Without values, a fixed width placeholder is used, so
/// that the width of the label does not follow the data.
fn stat_label(stat: &StatType, values: Option<(f64, f64, f64)>, print_extra_info: bool) -> String {
    let precision = stat.precision();
    let suffix = stat.metric();

    let (current, peak, avg) = match values {
        Some((current, peak, avg)) => (
            format!("{current:.*}", precision),
            format!("{peak:.*}", precision),
            format!("{avg:.*}", precision),
        ),
        None => {
            let placeholder = if precision > 0 { "00000.0" } else { "00000" }.to_owned();
            (placeholder.clone(), placeholder.clone(), placeholder)
        }
    };

    let mut label = format!("{}: {current}{suffix}", stat.display());
    if print_extra_info {
        if stat.show_peak() {
            write!(label, ", Peak {peak}{suffix}").unwrap();
        }

        write!(label, ", Avg {avg}{suffix}").unwrap();
    }

    label
}

impl RenderRequest {
    // Method to handle the actual drawing of the chart.
    pub fn draw(&self, cairo_context: &CairoContext) -> anyhow::Result<Vec<LegendLabel>> {
        let root = CairoBackend::new(cairo_context, (self.width, self.height))?.into_drawing_area();

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

        // Calculate the y-axis range from the interpolated points, since the
        // interpolation can overshoot the values present in the raw data.
        let series = data
            .iter()
            .map(|(stat_type, points)| (*stat_type, interpolate_series(points)))
            .collect::<Vec<_>>();

        let max_value = series
            .iter()
            .flat_map(|(_, points)| points.iter().map(|(_, value)| *value))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(cmp::Ordering::Equal))
            .unwrap_or_default();

        // Add some headroom above the highest point and round it up, so that the
        // series do not touch the border of the chart.
        let maximum_value = nice_upper_bound(max_value * Y_AXIS_HEADROOM).max(100.0);

        // The surface is transparent, so the card background and its rounded
        // corners show through it.
        // Scale the fonts with the size of the widget, otherwise the labels take
        // up most of the chart when it is rendered in a small window.
        // Follow the font of the UI, capped so that the labels always fit in the chart.
        let (_, chart_height) = root.dim_in_pixel();
        let max_font_size = f64::from((chart_height / 12).max(10));
        let axis_font_size = (self.base_font_size * 0.8).clamp(8.0, max_font_size) as u32;
        let title_font_size = (self.base_font_size * 1.2).clamp(9.0, max_font_size) as u32;

        let y_label_style = TextStyle {
            font: ("sans-serif", axis_font_size).into(),
            color: self.colors.text.to_backend_color(),
            pos: Pos::default(),
        };
        let y_label_area_size = root
            .estimate_text_size(&format!("{maximum_value}{value_suffix}"), &y_label_style)?
            .0
            + 10;

        // Build the legend entries first, they are needed to calculate the size of
        // the area reserved for the legend below the chart.
        let mut legend_entries = data
            .iter()
            .enumerate()
            .map(|(idx, (stat_type, points))| {
                let current = points.last().map(|(_, value)| *value).unwrap_or_default();
                let peak = points
                    .iter()
                    .map(|(_, value)| *value)
                    .reduce(f64::max)
                    .unwrap_or_default();
                let avg = points.iter().map(|(_, value)| *value).sum::<f64>() / points.len() as f64;

                LegendEntry {
                    label: stat_label(stat_type, Some((current, peak, avg)), self.print_extra_info),
                    color: Palette99::pick(idx).to_rgba(),
                }
            })
            .collect::<Vec<_>>();

        let throttling_sections = data_guard.throttling_sections();
        let has_throttling = throttling_sections
            .iter()
            .any(|section| !section.is_empty());
        let throttling_color = self.colors.throttling.mix(0.5);

        if has_throttling {
            let names: IndexSet<&str> = throttling_sections
                .iter()
                .flat_map(|section| section.iter().flat_map(|(_, values)| values))
                .map(|value| value.as_str())
                .collect();

            let label = format!(
                "Throttling: {}",
                names.into_iter().collect::<Vec<_>>().join(",")
            );
            legend_entries.push(LegendEntry {
                label,
                color: throttling_color,
            });
        }

        // Set up the main chart with axes and labels.
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(axis_font_size * 2)
            .y_label_area_size(y_label_area_size)
            .margin(10)
            .margin_top(20)
            .caption(
                self.title.as_str(),
                ("sans-serif", title_font_size, &self.colors.text),
            )
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
            .x_labels(4)
            .y_labels(6)
            .label_style(y_label_style.clone())
            .draw()
            .context("Failed to draw mesh")?;

        // Clip the series to the plotting area, otherwise points which are out of
        // the axis range are drawn over the axis labels and the chart title.
        let (x_range, y_range) = chart.plotting_area().get_pixel_range();
        cairo_context.save()?;
        cairo_context.rectangle(
            x_range.start as f64,
            y_range.start as f64,
            (x_range.end - x_range.start) as f64,
            (y_range.end - y_range.start) as f64,
        );
        cairo_context.clip();

        // Draw the main line series using cubic spline interpolation.
        for (entry, (_, data)) in legend_entries.iter().zip(&series) {
            chart
                .draw_series(LineSeries::new(
                    data.iter().copied(),
                    entry.color.stroke_width(2),
                ))
                .context("Failed to draw series")?;
        }

        // Draw throttling series
        if has_throttling {
            chart
                .draw_series(
                    throttling_sections
                        .iter()
                        .filter_map(|section| {
                            let first = section.first()?;
                            Some((
                                first.0,
                                section.last().map(|(ts, _)| *ts).unwrap_or(first.0),
                            ))
                        })
                        .map(|(start, end)| {
                            Rectangle::new(
                                [(start, 0f64), (end, maximum_value)],
                                throttling_color.filled(),
                            )
                        }),
                )
                .context("Failed to draw throttling histogram")?;
        }

        cairo_context.restore()?;

        root.present()?; // Present the final image.

        Ok(legend_entries
            .into_iter()
            .map(|entry| LegendLabel {
                text: entry.label,
                color: entry.color,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{nice_upper_bound, stat_label};

    #[test]
    fn test_nice_upper_bound() {
        assert_eq!(nice_upper_bound(61.6), 62.0);
        assert_eq!(nice_upper_bound(317.9), 320.0);
        assert_eq!(nice_upper_bound(2420.0), 2500.0);
    }

    #[test]
    fn test_stat_label() {
        let stat = crate::app::graphs_window::stat::StatType::Temperature("edge".into());

        assert_eq!(
            stat_label(&stat, Some((50.0, 60.0, 55.0)), true),
            "Temp (edge): 50.0\u{2103}, Peak 60.0\u{2103}, Avg 55.0\u{2103}"
        );
        assert_eq!(
            stat_label(&stat, None, true),
            "Temp (edge): 00000.0\u{2103}, Peak 00000.0\u{2103}, Avg 00000.0\u{2103}"
        );
    }
}
