mod arc;
mod constants;
mod enc;
mod pure;

use pyo3::prelude::*;

#[pymodule]
mod pd_proto {
    use crate::constants::FLOAT_DEFAULT_LIMIT;
    use crate::enc::*;
    use crate::pure::*;
    use pyo3::prelude::*;

    #[pyfunction]
    fn exponent(val: f64) -> usize {
        dec_places(val)
    }

    #[pyfunction]
    fn encode_varint(number: u64) -> Vec<u8> {
        let mut v = Vec::new();
        var_int(number, &mut v);
        v
    }

    #[pyfunction]
    fn encode_float(value: f64) -> Vec<u8> {
        let mut v = Vec::new();
        e_float(value, &mut v, FLOAT_DEFAULT_LIMIT);
        v
    }

    #[pyfunction]
    fn real_encrypt(
        py: Python<'_>,
        data: Bound<PyAny>,
        protocol_version: u8,
        max_depth: i32,
        float_limit: f64,
        string_length_limit: i32,
    ) -> PyResult<Vec<u8>> {
        enc(
            py,
            data,
            protocol_version,
            max_depth,
            string_length_limit,
            float_limit,
        )
    }
}
