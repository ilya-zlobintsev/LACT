use super::cubic_spline::cubic_spline_interpolation;
use super::to_texture_ext::ToTextureExt;
use super::PlotData;
use anyhow::Context;
use cairo::{Context as CairoContext, ImageSurface};

use gtk::gdk::MemoryTexture;
use itertools::Itertools;
use plotters::prelude::*;
use plotters::style::colors::full_palette::DEEPORANGE_100;
use plotters_cairo::CairoBackend;
use std::cmp::max;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use tracing::error;

enum Request {
    Terminate,
    Render(RenderRequest),
}

#[derive(Default)]
pub struct RenderRequest {
    pub title: String,
    pub value_suffix: String,
    pub secondary_value_suffix: String,
    pub y_label_area_size: u32,
    pub secondary_y_label_area_size: u32,

    pub data: PlotData,

    pub width: u32,
    pub height: u32,

    pub supersample_factor: u32,
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

        let thread_handle = std::thread::Builder::new()
            .name("Plot-Renderer".to_owned())
            // Render thread is very unimportant, skipping frames and rendering slowly is ok
            .spawn_with_priority(ThreadPriority::Min, {
                let state = state.clone();
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
                            // Create a new ImageSurface for Cairo rendering.
                            let mut surface = ImageSurface::create(
                                cairo::Format::ARgb32,
                                render_request.width as i32,
                                render_request.height as i32,
                            )
                            .unwrap();
                            let cairo_context = CairoContext::new(&surface).unwrap();

                            let cairo_backend = CairoBackend::new(
                                &cairo_context,
                                // Supersample the rendering
                                (
                                    render_request.width * render_request.supersample_factor,
                                    render_request.height * render_request.supersample_factor,
                                ),
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
                        // Terminate the thread if a Terminate request is received.
                        Some(Request::Terminate) => break,
                        None => {}
                    }
                }
        })
        .unwrap();

        Self {
            state,
            thread_handle: Some(thread_handle),
        }
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

        let data = &self.data;

        // Determine the start and end dates of the data series.
        let start_date = data
            .line_series_iter()
            .filter_map(|(_, data)| Some(data.first()?.0))
            .min()
            .unwrap_or_default();
        let end_date = data
            .line_series_iter()
            .map(|(_, value)| value)
            .filter_map(|data| Some(data.first()?.0))
            .max()
            .unwrap_or_default();

        // Calculate the maximum value for the y-axis.
        let mut maximum_value = data
            .line_series_iter()
            .flat_map(|(_, data)| data.iter().map(|(_, value)| value))
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap_or_default();

        // Ensure that the maximum value is at least 100 for better visualization.
        if maximum_value < 100.0f64 {
            maximum_value = 100.0f64;
        }

        root.fill(&WHITE)?; // Fill the background with white color.

        // Set up the main chart with axes and labels.
        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40 * self.supersample_factor)
            .y_label_area_size(self.y_label_area_size * self.supersample_factor)
            .right_y_label_area_size(self.secondary_y_label_area_size * self.supersample_factor)
            .margin(20 * self.supersample_factor)
            .caption(
                self.title.as_str(),
                ("sans-serif", 30 * self.supersample_factor),
            )
            .build_cartesian_2d(
                start_date..max(end_date, start_date + 60 * 1000),
                0f64..maximum_value,
            )?
            .set_secondary_coord(
                start_date..max(end_date, start_date + 60 * 1000),
                0.0..100.0,
            );

        // Configure the x-axis and y-axis mesh.
        chart
            .configure_mesh()
            .x_label_formatter(&|date_time| {
                let date_time = chrono::DateTime::from_timestamp_millis(*date_time).unwrap();
                date_time.format("%H:%M:%S").to_string()
            })
            .y_label_formatter(&|x| format!("{x}{}", &self.value_suffix))
            .x_labels(5)
            .y_labels(10)
            .label_style(("sans-serif", 30 * self.supersample_factor))
            .draw()
            .context("Failed to draw mesh")?;

        // Configure the secondary axes (for the secondary y-axis).
        chart
            .configure_secondary_axes()
            .y_label_formatter(&|x| format!("{x}{}", self.secondary_value_suffix.as_str()))
            .y_labels(10)
            .label_style(("sans-serif", 30 * self.supersample_factor))
            .draw()
            .context("Failed to draw mesh")?;

        // Draw the throttling histogram as a series of bars.
        chart
            .draw_series(
                data.throttling_iter()
                    // Group segments of consecutive enabled/disabled throttlings.
                    .chunk_by(|(_, _, point)| *point)
                    .into_iter()
                    // Only consider intervals where throttling is enabled.
                    .filter_map(|(point, group_iter)| point.then_some(group_iter))
                    // Get first and last times for the interval.
                    .filter_map(|mut group_iter| {
                        let first = group_iter.next()?;
                        Some((first, group_iter.last().unwrap_or(first)))
                    })
                    // Map the time intervals to rectangles representing throttling intervals.
                    .map(|((start, name, _), (end, _, _))| ((start, end), name))
                    .map(|((start_time, end_time), _)| (start_time, end_time))
                    // Sort by start_time to simplify merging.
                    .sorted_by_key(|&(start_time, _)| start_time)
                    // Merge overlapping or contiguous intervals.
                    .coalesce(|(start1, end1), (start2, end2)| {
                        if end1 >= start2 {
                            // Merge intervals by taking the earliest start_time and the latest end_time.
                            Ok((start1, std::cmp::max(end1, end2)))
                        } else {
                            // No overlap, keep the intervals separate.
                            Err(((start1, end1), (start2, end2)))
                        }
                    })
                    // Map the merged time intervals to rectangles.
                    .map(|(start_time, end_time)| {
                        Rectangle::new(
                            [(start_time, 0f64), (end_time, maximum_value)],
                            DEEPORANGE_100.filled(),
                        )
                    }),
            )
            .context("Failed to draw throttling histogram")?;

        // Draw the main line series using cubic spline interpolation.
        for (idx, (caption, data)) in (0..).zip(data.line_series_iter()) {
            chart
                .draw_series(LineSeries::new(
                    cubic_spline_interpolation(data.iter())
                        .into_iter()
                        .flat_map(|((first_time, second_time), segment)| {
                            // Interpolate in intervals of one millisecond.
                            (first_time..second_time).map(move |current_date| {
                                (current_date, segment.evaluate(current_date))
                            })
                        }),
                    Palette99::pick(idx).stroke_width(2 * self.supersample_factor), // Pick a unique color for the series.
                ))
                .context("Failed to draw series")?
                .label(caption) // Add label for the series
                .legend(move |(x, y)| {
                    let offset = 10 * self.supersample_factor as i32;
                    Rectangle::new(
                        [(x - offset, y - offset), (x + offset, y + offset)],
                        Palette99::pick(idx).filled(),
                    )
                });
        }

        // Draw the secondary line series on the secondary y-axis.
        for (idx, (caption, data)) in (0..).zip(data.secondary_line_series_iter()) {
            chart
                .draw_secondary_series(LineSeries::new(
                    cubic_spline_interpolation(data.iter())
                        .into_iter()
                        .flat_map(|((first_time, second_time), segment)| {
                            // Interpolate in intervals of one millisecond.
                            (first_time..second_time).map(move |current_date| {
                                (current_date, segment.evaluate(current_date))
                            })
                        }),
                    Palette99::pick(idx + 10).stroke_width(2 * self.supersample_factor), // Use a different color offset for secondary series.
                ))
                .context("Failed to draw series")?
                .label(caption) // Add label for secondary series.
                .legend(move |(x, y)| {
                    let offset = 10 * self.supersample_factor as i32;
                    Rectangle::new(
                        [(x - offset, y - offset), (x + offset, y + offset)],
                        Palette99::pick(idx + 10).filled(),
                    )
                });
        }

        // Configure and draw series labels (the legend).
        chart
            .configure_series_labels()
            .margin(40 * self.supersample_factor)
            .label_font(("sans-serif", 30 * self.supersample_factor))
            .position(SeriesLabelPosition::LowerRight)
            .legend_area_size(15 * self.supersample_factor as i32)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .context("Failed to draw series labels")?;

        root.present()?; // Present the final image.
        Ok(())
    }
}
