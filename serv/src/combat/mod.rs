use comn::{prelude::*, vec_of_pos};
use pyo3::{prelude::*, types::PyAny};
use specs::{prelude::*, Component};
use std::string::ToString;

mod alignment;
pub use alignment::Alignment;

#[pyclass]
#[derive(Debug, Clone, Component)]
/// Entities with this component will chase other entities
/// who have an Alignment component that matches the one in
/// the `target` field.
/// In order for the chasing to begin and/or continue, these
/// entities must also be within the distance indicated in the
/// `distance` field.
pub struct Chaser {
    /// Entities
    pub target: Alignment,
    #[pyo3(get, set)]
    pub distance_squared: f32,
    #[pyo3(get, set)]
    pub speed: f32,
}
#[pymethods]
impl Chaser {
    #[new]
    fn new(obj: &PyRawObject, target: String, distance: f32, speed: f32) {
        obj.init(Self {
            target: Alignment::str(&target).unwrap(),
            distance_squared: distance.powi(2),
            speed,
        })
    }

    #[getter]
    fn get_target(&self) -> String {
        self.target.to_string()
    }
    #[setter]
    fn set_target(&mut self, value: String) {
        self.target = Alignment::str(&value).unwrap();
    }
}

impl<'source> FromPyObject<'source> for Chaser {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        unsafe {
            let py = Python::assume_gil_acquired();
            let obj = ob.to_object(py);
            Ok(Self {
                speed: obj.getattr(py, "speed")?.extract(py)?,
                distance_squared: obj.getattr(py, "distance")?.extract::<f32>(py)?.powi(2),
                target: obj.getattr(py, "target")?.extract(py)?,
            })
        }
    }
}

pub struct Chase;
impl<'a> System<'a> for Chase {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Pos>,
        ReadStorage<'a, Alignment>,
        ReadStorage<'a, Chaser>,
    );

    fn run(&mut self, (ents, mut poses, aligns, chasers): Self::SystemData) {
        use na::Translation2;
        (&chasers, &poses, &ents)
            .join()
            // returns (chaser ent, direction to go in)
            .filter_map(|(chaser, vec_of_pos!(chaser_vec), chaser_ent)| {
                (&poses, &aligns)
                    .join()
                    .filter_map(|(vec_of_pos!(chased_vec), align)| {
                        if align == &chaser.target {
                            let delta = chased_vec - chaser_vec;
                            let delta_distance_squared = delta.magnitude_squared();
                            if delta_distance_squared < chaser.distance_squared {
                                return Some((delta, delta_distance_squared));
                            }
                        }
                        None
                    })
                    // finds the entity with the shortest distance from the chaser
                    .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(&dist_b).unwrap())
                    .map(|(delta, _)| (chaser_ent, delta))
            })
            .collect::<Vec<_>>()
            .into_iter()
            // move the chaser in the specified direction
            .for_each(|(chaser_ent, dir)| {
                let chaser = chasers.get(chaser_ent).unwrap();
                let chaser_pos = poses.get_mut(chaser_ent).unwrap();
                chaser_pos.0.translation.vector += dir.normalize() * chaser.speed;
            });
    }
}
