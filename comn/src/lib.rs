#![feature(stmt_expr_attributes)]

pub use nalgebra as na;
pub use ncollide2d as collide;
pub use rmp_serde as rmps;
pub use serde;
pub use specs;

pub mod prelude {
    pub use super::na;
    pub use super::specs;
    pub use super::{Dead, Iso2, Item, Pos, Vec2};
}
use prelude::*;
#[cfg(feature = "python")]
use pyo3::{prelude::*, types::PyAny};
use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};

#[cfg(feature = "python")]
pub trait PyWrapper<C: std::fmt::Debug + Component + Send + Sync> {
    fn into_inner(self) -> C;
}

pub type Vec2 = na::Vector2<f32>;
pub type Iso2 = na::Isometry2<f32>;
pub use collide::shape::Cuboid;

#[cfg(feature = "python")]
#[pyclass(name=Iso2)]
#[derive(Debug, Clone)]
pub struct PyIso2 {
    #[pyo3(get, set)]
    loc: PyVec2,
    #[pyo3(get, set)]
    rot: f32,
}
#[cfg(feature = "python")]
#[pymethods]
impl PyIso2 {
    #[new]
    fn new(obj: &PyRawObject, loc: PyVec2, rot: f32) {
        obj.init(Self { loc, rot })
    }
}
#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for PyIso2 {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj: PyObject = ob.to_object(py);
            Ok(Self {
                loc: obj.getattr(py, "loc")?.extract(py)?,
                rot: obj.getattr(py, "rot")?.extract(py)?,
            })
        }
    }
}
#[cfg(feature = "python")]
impl Into<Iso2> for PyIso2 {
    fn into(self) -> Iso2 {
        use na::{Translation, UnitComplex};
        Iso2::from_parts(
            Translation::from(self.loc.inner),
            UnitComplex::new(self.rot),
        )
    }
}
#[cfg(feature = "python")]
impl PyIso2 {
    pub fn from_iso2(iso: Iso2) -> Self {
        Self {
            loc: PyVec2 {
                inner: iso.translation.vector,
            },
            rot: iso.rotation.angle(),
        }
    }
}

#[cfg(feature = "python")]
#[derive(Debug, Clone)]
pub struct PyVec2 {
    inner: Vec2,
}
#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for PyVec2 {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let s: Vec<f32> = ob.extract()?;
        Ok(Self {
            inner: Vec2::from_vec(s),
        })
    }
}
#[cfg(feature = "python")]
impl IntoPy<PyObject> for PyVec2 {
    fn into_py(self, py: Python) -> PyObject {
        let v = self.inner;
        vec![v.x, v.y].into_py(py)
    }
}

#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug, Component, Serialize, Deserialize)]
pub struct Pos {
    pub iso: Iso2,
}
#[cfg(feature = "python")]
#[pymethods]
impl Pos {
    #[new]
    fn new(obj: &PyRawObject, py_iso: PyIso2) {
        obj.init(Self { iso: py_iso.into() });
    }

    #[getter]
    fn get_iso(&self) -> PyIso2 {
        PyIso2::from_iso2(self.iso)
    }
    #[setter]
    fn set_iso(&mut self, py_iso: PyIso2) {
        self.iso = py_iso.into();
    }
}
#[cfg(not(feature = "python"))]
#[derive(Clone, Debug, Component, Serialize, Deserialize)]
pub struct Pos {
    pub iso: Iso2,
}

impl Pos {
    pub fn vec(vec: Vec2) -> Self {
        Pos {
            iso: Iso2::new(vec, na::zero()),
        }
    }
}

#[macro_export]
macro_rules! vec_of_pos {
    ($($vec_name:tt)*) => {
        Pos { iso: Iso2 {
            translation: Translation2 { vector: $($vec_name)* },
            ..
        } }
    }
}
#[macro_export]
macro_rules! trans_of_pos {
    ($($trans_name:tt)*) => {
        Pos { iso: Iso2 {
            translation: $($trans_name)* ,
            ..
        } }
    }
}

#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone, Debug, Component, Serialize, Deserialize)]
pub struct Hitbox {
    pub cuboid: Cuboid<f32>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Hitbox {
    #[new]
    fn new(obj: &PyRawObject, extents: PyVec2) {
        obj.init(Self {
            cuboid: Cuboid::new(extents.inner / 2.0),
        })
    }

    #[getter]
    fn get_extents(&self) -> PyVec2 {
        let h = self.cuboid.half_extents();
        PyVec2 { inner: h * 2.0 }
    }
    #[setter]
    fn set_extents(&mut self, extents: PyVec2) {
        self.cuboid = Cuboid::new(extents.inner / 2.0);
    }
}

#[cfg(not(feature = "python"))]
#[derive(Clone, Debug, Component, Serialize, Deserialize)]
pub struct Hitbox {
    pub cuboid: Cuboid<f32>,
}

impl Hitbox {
    /// Create a Hitbox from its half extents.
    /// (i.e. it will be twice as large as the provided numbers on each axis)
    pub fn vec(vec: Vec2) -> Self {
        Self {
            cuboid: Cuboid::new(vec),
        }
    }
}

#[derive(Default)]
pub struct Fps(pub f32);

pub mod art;

pub mod combat;

pub mod item;
pub use item::Item;

pub mod dead;
pub use dead::Dead;

pub mod controls;

pub mod phys;

pub mod net {
    pub use comp::NetComponent;
    pub use msg::NetMessage;
    // UpdatePosition
    use super::prelude::*;
    use serde::{Deserialize, Serialize};
    use specs::{prelude::*, Component};

    #[derive(Clone, Debug, Component, Serialize, Deserialize)]
    /// These wrap around an Iso2.
    /// They're sent from the Server to the Client
    /// to update positions, no entity on the Server
    /// should have one of those, though they should
    /// be fairly common on the Client.
    pub struct UpdatePosition {
        pub iso: Iso2,
        // duration since UNIX_EPOCH
        pub time_stamp: std::time::Duration,
    }

    #[derive(Clone, Debug, Component, Serialize, Deserialize)]
    /// This is sent in by the player when they're ready
    /// for their Pos and Appearance components.
    /// Essentially, when they want to enter the game world.
    /// Menu/Spectator -> Game
    pub struct SpawnPlayer;

    #[derive(Clone, Debug, Component, Serialize, Deserialize)]
    /// The server attaches this to an entity on the clients to
    /// tell clients which entity they are able to control.
    pub struct LocalPlayer;

    mod msg {
        use super::NetComponent;
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize, Serialize, Debug)]
        /// All possible messages that can be sent between the client and server.
        pub enum NetMessage {
            /// Instructs the client to create a new entity.
            /// This is also internally sent from the client to the server
            /// to establish the connection. If it's sent after the connection
            /// is established, it's simply ignored.
            NewEnt(u32),

            /// Inserts (possibly overwriting an existing component) a component
            /// on the client. On the server, the `u32` is ignored, and components
            /// can only be inserted onto the client that requested them.
            InsertComp(u32, NetComponent),

            /// Contains all of the important data necessary to connect a new client to the game.
            /// If it's sent from the client to the server, it's ignored.
            Establishment {
                /// Tells the local client which of the entities they are.
                local_player: u32,
                /// A record of which indexes refer to which appearance names.
                appearance_record: crate::art::AppearanceRecord,
            },
        }
    }

    mod comp {
        // util includes
        use crate::Pos;
        use serde::{Deserialize, Serialize};
        use specs::{Entity, LazyUpdate};

        macro_rules! net_component_base {
            ( $( $x:tt : $y:ty $(: $extra:ident)? ),+ $(,)? ) => {
                #[derive(Deserialize, Serialize, Debug)]
                pub enum NetComponent {
                    $(
                        $x($y),
                    )+
                }

                $(
                    impl From<$y> for NetComponent {
                        fn from(c: $y) -> Self {
                            NetComponent::$x(c)
                        }
                    }
                )+

                impl NetComponent {
                    pub fn insert(self, ent: Entity, lu: &LazyUpdate) {
                        match self {
                            $(
                                NetComponent::$x(c) => lu.insert(ent, c),
                            )+
                        }
                    }
                }
            };
        }

        macro_rules! net_component {
            ( $( $name:ident $(: $inner:ty)? ),+ $(,)? ) => {
                net_component_base! {
                    $($name $(: $inner)? : $name),*
                }
            }
        }

        // Component includes
        use super::{LocalPlayer, SpawnPlayer, UpdatePosition};
        use crate::art::{Animate, Appearance, PlayerAnimationController};
        use crate::combat::AttackRequest;
        use crate::controls::{Camera, Heading, Speed};
        use crate::dead::Dead;
        use crate::item::{Deposition, DropRequest, Inventory, PickupRequest};
        use crate::{Hitbox, Item};

        net_component! {
            // art
            Appearance,
            Animate,
            PlayerAnimationController,

            // inventory
            Item,
            Deposition,
            Inventory,
            PickupRequest,
            DropRequest,

            // phys/net
            Pos,
            Hitbox,
            UpdatePosition,
            SpawnPlayer,
            LocalPlayer,
            Heading,
            Speed,
            Camera,

            // uhh yeah
            Dead,
            AttackRequest,
        }
    }
}
pub use net::{NetComponent, NetMessage};
