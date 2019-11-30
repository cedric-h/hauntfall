use comn::prelude::*;
use comn::strum::{Display, EnumIter, EnumString};
use pyo3::{prelude::*, types::PyAny};
use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};
use strum::IntoEnumIterator;

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
