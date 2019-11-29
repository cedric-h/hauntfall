use comn::prelude::*;
use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Component)]
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
