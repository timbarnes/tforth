// tForth utilities

pub fn is_integer(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

pub fn is_float(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests {

    use crate::utility::*;
    #[test]
    fn is_int1() {
        assert!(is_integer("42"));
    }
    #[test]
    fn is_int2() {
        assert!(is_integer("-24"));
    }
    #[test]
    fn is_int3() {
        assert!(!is_integer("2.5"));
    }
    #[test]
    fn is_int4() {
        assert!(!is_integer("0b000"));
    }
    #[test]
    fn is_int() {
        assert!(!is_integer("blah"));
    }

    #[test]
    fn is_float1() {
        assert!(is_float("42.0"));
    }
    #[test]
    fn is_float2() {
        assert!(is_float("-2.4"));
    }
    #[test]
    fn is_float3() {
        assert!(is_float("2"));
    }
    #[test]
    fn is_float4() {
        assert!(!is_float("0b000"));
    }
    #[test]
    fn is_float5() {
        assert!(!is_float("blah"));
    }
}
