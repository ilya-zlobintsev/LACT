#[derive(Clone, Copy)]
pub struct CubicSplineSegment {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    x: i64,
}

impl CubicSplineSegment {
    // Create a new cubic spline segment
    fn new(a: f64, b: f64, c: f64, d: f64, x: i64) -> Self {
        Self { a, b, c, d, x }
    }

    // Evaluate the cubic spline at a given point x
    pub fn evaluate(&self, x: i64) -> f64 {
        let dx = (x - self.x) as f64;
        self.a + self.b * dx + self.c * dx.powi(2) + self.d * dx.powi(3)
    }
}

pub type TimePeriod = (i64, i64);

// Define a function to perform cubic spline interpolation
pub fn cubic_spline_interpolation(
    data: &[(i64, f64)],
) -> impl Iterator<Item = (TimePeriod, CubicSplineSegment)> + '_ {
    let n = data.len();

    // Compute differences between consecutive x values
    let mut dx = Vec::with_capacity(n - 1);
    for i in 1..n {
        let x_diff = (data[i].0 - data[i - 1].0) as f64;
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

    let mut iter = data
        .iter()
        .zip(
            // Construct cubic spline segments
            (0..(n - 1)).map(move |i| {
                let a = dy[i];
                let b = (dy[i + 1] - dy[i]) / dx[i] - dx[i] * (2.0 * d[i] + d[i + 1]) / 6.0;
                let c = d[i] / 2.0;
                let d = (d[i + 1] - d[i]) / (6.0 * dx[i]);
                CubicSplineSegment::new(a, b, c, d, data[i].0)
            }),
        )
        .peekable();

    std::iter::repeat(()).map_while(move |_| {
        let ((first_time, _), segment) = iter.next()?;
        let ((second_time, _), _) = iter.peek()?;
        Some(((*first_time, *second_time), segment))
    })
}
