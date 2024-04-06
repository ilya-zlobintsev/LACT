use chrono::offset::Local;
use chrono::DateTime;
use itertools::Itertools;

#[derive(Clone, Copy)]
pub struct CubicSplineSegment {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    x: DateTime<Local>,
}

impl CubicSplineSegment {
    // Create a new cubic spline segment
    fn new(a: f64, b: f64, c: f64, d: f64, x: DateTime<Local>) -> Self {
        Self { a, b, c, d, x }
    }

    // Evaluate the cubic spline at a given point x
    pub fn evaluate(&self, x: &DateTime<Local>) -> f64 {
        let dx = (*x - self.x).num_milliseconds() as f64;
        self.a + self.b * dx + self.c * dx.powi(2) + self.d * dx.powi(3)
    }
}

pub type TimePeriod = (DateTime<Local>, DateTime<Local>);

// Define a function to perform cubic spline interpolation
pub fn cubic_spline_interpolation<'a, I>(iter: I) -> Vec<(TimePeriod, CubicSplineSegment)>
where
    I: IntoIterator<Item = (&'a DateTime<Local>, &'a f64)> + 'a,
{
    let data: Vec<_> = iter.into_iter().collect();

    let n = data.len();

    // Compute differences between consecutive x values
    let mut dx = Vec::with_capacity(n - 1);
    for i in 1..n {
        let x_diff = (*data[i].0 - *data[i - 1].0).num_milliseconds() as f64;
        dx.push(x_diff);
    }

    // Compute differences between consecutive y values
    let dy: Vec<f64> = data.iter().map(|&(_, y)| *y).collect();

    // Compute second derivatives using Thomas algorithm
    let mut d = vec![0.0; n];
    let mut s = vec![0.0; n - 1];
    let mut q = vec![0.0; n];
    let mut p = vec![0.0; n];

    for i in 1..(n - 1) {
        s[i] = dx[i - 1] / (dx[i - 1] + dx[i]);
        q[i] = 1.0 - s[i];
        p[i] = 6.0 * ((dy[i + 1] - dy[i]) / dx[i] - (dy[i] - dy[i - 1]) / dx[i - 1])
            / (dx[i - 1] + dx[i]);
    }

    for i in 1..(n - 1) {
        let temp = q[i] * d[i - 1] + 2.0;
        d[i] = -s[i] / temp;
        p[i] = (p[i] - q[i] * p[i - 1]) / temp;
    }

    for i in (1..(n - 1)).rev() {
        d[i] = d[i] * d[i + 1] + p[i];
    }

    data.iter()
        .zip(
            // Construct cubic spline segments
            (0..(n - 1)).map(|i| {
                let a = dy[i];
                let b = (dy[i + 1] - dy[i]) / dx[i] - dx[i] * (2.0 * d[i] + d[i + 1]) / 6.0;
                let c = d[i] / 2.0;
                let d = (d[i + 1] - d[i]) / (6.0 * dx[i]);
                CubicSplineSegment::new(a, b, c, d, *data[i].0)
            }),
        )
        // Group 2 closest points together
        .tuple_windows::<(_, _)>()
        // Get first time, second time and their corresponding interpolation segment
        .map(|(((first_time, _), segment), ((second_time, _), _))| {
            ((**first_time, **second_time), segment)
        })
        .collect()
}
