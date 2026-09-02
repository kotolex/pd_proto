mod pure;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod pd_proto {
    use crate::pure::*;
    use pyo3::prelude::*;
    #[pyfunction]
    fn exponent(val: f64) -> usize {
        dec_places(val)
    }
}
