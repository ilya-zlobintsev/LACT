use chrono::offset::Local;
use chrono::DateTime;

// Define a struct to represent a cubic spline segment
pub struct CubicSplineSegment {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    pub x: DateTime<Local>,
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

// Define a function to perform cubic spline interpolation
pub fn cubic_spline_interpolation(data: &[(DateTime<Local>, f64)]) -> Vec<CubicSplineSegment> {
    let n = data.len();

    // Compute differences between consecutive x values
    let mut dx = Vec::with_capacity(n - 1);
    for i in 1..n {
        let x_diff = (data[i].0 - data[i - 1].0).num_milliseconds() as f64;
        dx.push(x_diff);
    }

    // Compute differences between consecutive y values
    let dy: Vec<f64> = data.iter().map(|&(_, y)| y).collect();

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

    // Construct cubic spline segments
    let mut segments = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let a = dy[i];
        let b = (dy[i + 1] - dy[i]) / dx[i] - dx[i] * (2.0 * d[i] + d[i + 1]) / 6.0;
        let c = d[i] / 2.0;
        let d = (d[i + 1] - d[i]) / (6.0 * dx[i]);
        segments.push(CubicSplineSegment::new(a, b, c, d, data[i].0));
    }

    segments.sort_by(|a, b| a.x.cmp(&b.x));

    segments
}
