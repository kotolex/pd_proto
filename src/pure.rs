pub fn dec_places(val: f64) -> usize {
    let s = val.to_string();
    if let Some(pos) = s.find('.') {
        s[pos + 1..].trim_end_matches('0').len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dec_places() {
        assert_eq!(dec_places(123.456), 3);
        assert_eq!(dec_places(7.0), 0);
        assert_eq!(dec_places(0.0001), 4);
        assert_eq!(dec_places(12.123000), 3);
    }
}
