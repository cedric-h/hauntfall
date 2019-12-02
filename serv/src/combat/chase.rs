use comn::controls::Heading;
use comn::{prelude::*, vec_of_pos};
use pyo3::prelude::*;
use specs::{prelude::*, Component};

pub use super::alignment::{Alignment, PyAlignment};

#[pyclass]
#[derive(Debug, Clone, Component)]
/// Entities with this component will chase other entities
/// who have an Alignment component that matches the one in
/// the `target` field.
/// In order for the chasing to begin and/or continue, these
/// entities must also be within the distance indicated in the
/// `distance` field.
pub struct Chaser {
    #[pyo3(get, set)]
    pub target: Alignment,
    /// This is squared so that physics distance calculations
    /// can be faster. It's exposed to the Python API as if it
    /// weren't squared.
    /// Instead, distance is automatically squared/sqaure rooted when
    /// touched by Python, so that the performance impact is felt only when
    /// Python meddles, not every frame for every entity when the chasing
    /// calculations are done.
    pub distance_squared: f32,
}
#[pymethods]
impl Chaser {
    #[new]
    fn new(obj: &PyRawObject, target: PyAlignment, distance: f32) {
        obj.init(Self {
            target: target.inner,
            distance_squared: distance.powi(2),
        })
    }

    #[getter]
    fn get_distance(&self) -> f32 {
        self.distance_squared.sqrt()
    }
    #[setter]
    fn set_distance(&mut self, distance: f32) {
        self.distance_squared = distance.powi(2);
    }
}

pub struct Chase;
impl<'a> System<'a> for Chase {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Heading>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Alignment>,
        ReadStorage<'a, Chaser>,
    );

    fn run(&mut self, (ents, mut headings, poses, aligns, chasers): Self::SystemData) {
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
                headings
                    .insert(
                        chaser_ent,
                        Heading {
                            dir: na::Unit::new_normalize(dir),
                        },
                    )
                    .expect("Couldn't update Heading for Chaser");
            });
    }
}
