use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction()]
#[pyo3(text_signature = "(a, b)")]
/// Return the sum of the two input integers
///
/// Arguments
/// ---------
/// a: int,
///     The first integer addend
/// b: int
///     The second integer addend
pub fn add_two_numbers(a: u64, b: u64) -> u64 {
    a + b
}

#[pymodule]
fn simple_pkg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(add_two_numbers))?;
    Ok(())
}