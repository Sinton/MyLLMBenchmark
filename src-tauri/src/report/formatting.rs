pub(crate) fn trim_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value:.2}")
    }
}

pub fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
