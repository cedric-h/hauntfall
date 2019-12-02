use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Component)]
pub struct AttackRequest;

#[cfg(feature = "python")]
#[pyclass]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Component)]
pub struct Health {
    #[pyo3(get, set)]
    pub current: u32,
    #[pyo3(get, set)]
    pub max: u32,
}
#[cfg(feature = "python")]
#[pymethods]
impl Health {
    #[new]
    fn new(obj: &PyRawObject, max: u32, current: u32) {
        obj.init(Self { max, current })
    }
}

#[cfg(not(feature = "python"))]
#[derive(Serialize, Deserialize, Debug, Clone, Default, Component)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    pub fn full(max: u32) -> Self {
        Self {
            current: max.clone(),
            max,
        }
    }
}

mod damage {
    use crate::prelude::*;
    use specs::prelude::*;
    use specs::Component;

    #[derive(Clone, Debug, Component)]
    pub struct Damage {
        pub knockback: Vec2,
        pub hp: u32,
    }
}
pub use damage::Damage;

mod hurtbox {
    use super::Damage;
    #[cfg(feature = "python")]
    use pyo3::prelude::*;
    use crate::prelude::*;
    use specs::prelude::*;
    use specs::Component;

    #[cfg(feature = "python")]
    #[pyclass]
    #[derive(Debug, Clone, Component)]
    /// How much someone gets hurt if they get in
    /// this thing's way when it's going somewhere.
    pub struct Hurtbox {
        hp: u32,
        knockback: f32,
    }
    #[cfg(feature = "python")]
    #[pymethods]
    impl Hurtbox {
        #[new]
        fn new(obj: &PyRawObject, hp: u32, knockback: f32) {
            obj.init(Self { hp, knockback })
        }
    }

    #[cfg(not(feature = "python"))]
    #[derive(Debug, Clone, Component)]
    pub struct Hurtbox {
        hp: u32,
        knockback: f32,
    }

    impl Hurtbox {
        pub fn into_damage(self, knockback: &Vec2) -> Damage {
            Damage {
                hp: self.hp,
                knockback: knockback * self.knockback,
            }
        }
    }
}
pub use hurtbox::Hurtbox;
