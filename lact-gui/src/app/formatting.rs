use std::fmt;

/// Displays numeric values with monospace font.
/// Should be used in oftent updated data.
pub enum Mono {
    #[allow(dead_code)]
    Int(i64),
    UInt(u64),
    Float {
        value: f64,
        precision: usize,
    },
}

impl Mono {
    #[allow(dead_code)]
    pub fn int(value: impl Into<i64>) -> Self {
        Self::Int(value.into())
    }

    pub fn uint(value: impl Into<u64>) -> Self {
        Self::UInt(value.into())
    }

    pub fn float(value: impl Into<f64>, precision: usize) -> Self {
        Self::Float {
            value: value.into(),
            precision,
        }
    }
}

impl fmt::Display for Mono {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<span font_family='monospace'>")?;
        match *self {
            Self::Int(v) => write!(f, "{v}")?,
            Self::UInt(v) => write!(f, "{v}")?,
            Self::Float { value, precision } => write!(f, "{value:.prec$}", prec = precision)?,
        }
        f.write_str("</span>")
    }
}
