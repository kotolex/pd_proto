mod arc;
mod constants;
mod dec;
mod enc;
mod pure;

use pyo3::prelude::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[pymodule]
mod pd_proto {
    use crate::constants::FLOAT_DEFAULT_LIMIT;
    use crate::dec::*;
    use crate::enc::*;
    use pyo3::prelude::*;

    #[pyfunction]
    fn decode_varint(buffer: Vec<u8>, offset: usize) -> PyResult<(u64, usize)> {
        d_varint(&buffer, offset)
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

    #[pyfunction]
    fn real_decrypt(
        py: Python<'_>,
        buffer: Vec<u8>,
        offset: usize,
    ) -> PyResult<(Bound<'_, PyAny>, usize)> {
        r_decrypt_base(py, buffer, offset)
    }
}
