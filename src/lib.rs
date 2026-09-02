use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod pd_proto {
    use pyo3::prelude::*;
    #[pyfunction]
    fn exponent(val: f64) -> usize {
        let s = val.to_string();
        if let Some(pos) = s.find('.') {
            s[pos + 1..].trim_end_matches('0').len()
        } else {
            0
        }
    }
}
