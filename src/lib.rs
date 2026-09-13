mod constants;
mod dec;
mod enc;
mod options;
mod utils;

use pyo3::prelude::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc; // should speed up allocations in Linux

#[pymodule]
mod pd_proto {
    use crate::dec::{decode_varint as dec_var, unpack as real_unpack};
    use crate::enc::{encode_float as e_float, encode_varint as var_int, pack as real_pack};
    use crate::options::Options;
    use pyo3::prelude::*;

    #[pyfunction]
    fn decode_varint(buffer: Vec<u8>, offset: usize) -> PyResult<(u64, usize)> {
        dec_var(&buffer, offset)
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
        let mut opts = Options::new_default();
        e_float(value, &mut v, &mut opts);
        v
    }

    #[pyfunction]
    fn pack(
        py: Python<'_>,
        data: Bound<PyAny>,
        protocol_version: u8,
        max_depth: i32,
        float_limit: f64,
        string_length_limit: i32,
    ) -> PyResult<Vec<u8>> {
        real_pack(
            py,
            data,
            protocol_version,
            max_depth,
            string_length_limit,
            float_limit,
        )
    }

    #[pyfunction]
    fn unpack(
        py: Python<'_>,
        buffer: Vec<u8>,
        offset: usize,
        max_depth: i32,
    ) -> PyResult<(Bound<'_, PyAny>, usize)> {
        real_unpack(py, buffer, offset, max_depth)
    }
}
