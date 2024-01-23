// Tforth utilities

pub fn is_number(s: &str) -> bool {
    s.parse::<i32>().is_ok()
}
