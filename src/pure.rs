pub fn dec_places(f: f64) -> usize {
    let cv = f.to_string();
    if let Some((_, pos)) = cv.split_once(".") {
        let mut index = pos.len();
        while pos[index..] == *"0" {
            index -= 1
        }
        pos[0..index].len()
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
