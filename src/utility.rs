// Tforth utilities

pub fn is_integer(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

pub fn is_float(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}
