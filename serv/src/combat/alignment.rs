use comn::prelude::*;
use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};
// scripting
use pyo3::{prelude::*, types::PyAny};
// string enum
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[pyclass(name=Alignment)]
#[derive(Debug, Clone, Component)]
pub struct PyAlignment {
    #[pyo3(get, set)]
    pub inner: Alignment,
}
#[pymethods]
impl PyAlignment {
    #[new]
    fn new(obj: &PyRawObject, inner: Alignment) {
        obj.init(Self { inner })
    }
}
impl<'source> FromPyObject<'source> for PyAlignment {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj: PyObject = ob.to_object(py);
            Ok(Self {
                inner: obj.getattr(py, "inner")?.extract(py)?,
            })
        }
    }
}
impl comn::PyWrapper<Alignment> for PyAlignment {
    fn into_inner(self) -> Alignment {
        self.inner
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, EnumIter, Serialize, Deserialize, Component)]
/// The alignment of a particular Entity;
/// which team it's on.
///
/// # Equivalency
/// `All` will always be equivalent to the `Alignment` it's being
/// compared to, even if that `Alignment` is `All` or `Neither`.
/// `Neither` will never be equivalent to another `Alignment`,
/// including `Neither`, unless that `Alignment` is `All`.
pub enum Alignment {
    Players,
    Enemies,
    All,
    Neither,
}
impl<'source> FromPyObject<'source> for Alignment {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let s: &str = ob.extract()?;
        Ok(Alignment::str(s).unwrap())
    }
}
impl IntoPy<PyObject> for Alignment {
    fn into_py(self, py: Python) -> PyObject {
        self.to_string().into_py(py)
    }
}
impl Alignment {
    #[inline]
    pub fn str(s: &str) -> Result<Self, String> {
        use std::str::FromStr;
        Alignment::from_str(s).map_err(|_| {
            format!(
                "{} is not an Alignment! Alignment must be one of: {:?}",
                s,
                Alignment::iter().collect::<Vec<_>>(),
            )
        })
    }
}
impl PartialEq for Alignment {
    fn eq(&self, other: &Self) -> bool {
        use Alignment::*;

        match self {
            All => true,
            Neither => false,
            &x => match other {
                All => true,
                Neither => false,
                &y => x as usize == y as usize,
            },
        }
    }
}

#[test]
fn test_alignment() {
    pub use Alignment::*;

    for align in [Players, Enemies].iter() {
        assert!(*align == All);
        assert!(*align != Neither);
        assert!(*align == Enemies || *align == Players);
    }

    assert!(All == All);
    assert!(All == Neither);
    assert!(Neither != Neither);
}
