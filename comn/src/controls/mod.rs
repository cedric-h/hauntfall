use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};

pub mod movement;
pub use movement::MoveHeadings;

#[derive(Clone, Debug, Component, Serialize, Deserialize)]
/// Nobody gets these on the Server, but the Server
/// will tell the Client to put one on the entity the Client
/// is looking out of at the moment.
/// NOTE: This isn't used atm.
pub struct Camera;

mod moving {
    #[cfg(feature = "python")]
    use pyo3::prelude::*;
    #[cfg(feature = "python")]
    use crate::PyVec2;
    use crate::prelude::*;
    use serde::{Deserialize, Serialize};
    use specs::{prelude::*, Component};

    #[cfg(feature = "python")]
    #[pyclass]
    #[derive(Clone, Debug, Component, Serialize, Deserialize)]
    /// How much should we move your Heading?
    pub struct Speed {
        #[pyo3(get, set)]
        pub speed: f32
    }
    #[cfg(feature = "python")]
    #[pymethods]
    impl Speed {
        #[new]
        fn new(obj: &PyRawObject, speed: f32) {
            obj.init(Self { speed })
        }
    }
    #[cfg(not(feature = "python"))]
    #[derive(Clone, Debug, Component, Serialize, Deserialize)]
    /// How much should we move your Heading?
    pub struct Speed {
        pub speed: f32
    }

    #[cfg(feature = "python")]
    #[pyclass]
    #[derive(Clone, Debug, Component, Serialize, Deserialize)]
    /// Where would the Client like to go?
    /// Note that the server isn't necessarily going to actually get them there.
    pub struct Heading {
        pub dir: na::Unit<Vec2>,
    }
    #[cfg(feature = "python")]
    #[pymethods]
    impl Heading {
        #[new]
        fn new(obj: &PyRawObject, speed: PyVec2) {
            obj.init(Self { dir: na::Unit::new_normalize(speed.inner) })
        }
    }
    #[cfg(not(feature = "python"))]
    #[derive(Clone, Debug, Component, Serialize, Deserialize)]
    pub struct Heading {
        pub dir: na::Unit<Vec2>,
    }
}
pub use moving::{Speed, Heading};
