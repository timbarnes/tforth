// Tforth utilities

pub fn is_integer(s: &str) -> bool {
    s.parse::<i32>().is_ok()
}

pub fn is_float(s: &str) -> bool {
    s.parse::<f32>().is_ok()
}
