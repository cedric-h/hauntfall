use serde::{Deserialize, Serialize};
use specs::{prelude::*, Component};
use std::fmt::Debug;

pub mod player_anim;
pub use player_anim::PlayerAnimationController;

#[derive(Clone, Debug, Default, Component, Serialize, Deserialize)]
/// Entities with this component are rendered at a special stage on the client,
/// and their origin is in the (center, center) rather than their (center, bottom)
pub struct Animate {
    pub current_frame: usize,
    pub row: usize,
}

impl Animate {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            row: 0,
        }
    }
    pub fn row(row: usize) -> Self {
        Self {
            current_frame: 0,
            row,
        }
    }
}

pub struct UpdateAnimations;
impl<'a> System<'a> for UpdateAnimations {
    type SystemData = (WriteStorage<'a, Animate>, ReadStorage<'a, Appearance>);

    fn run(&mut self, (mut animates, appearances): Self::SystemData) {
        for (animate, appearance) in (&mut animates, &appearances).join() {
            let SpritesheetData { rows, .. } = crate::art::SPRITESHEETS
                .get(appearance)
                .unwrap_or_else(|| panic!("No animation data found for {:?}!", appearance));

            let AnimationData {
                total_frames,
                frame_duration,
            } = rows
                .get(animate.row)
                .unwrap_or_else(|| panic!("{:?} has no row #{}!", appearance, animate.row));

            animate.current_frame += 1;

            // greater than or equal to because it starts at 0
            if animate.current_frame >= total_frames * frame_duration {
                animate.current_frame = 0;
            }
        }
    }
}

#[derive(Clone)]
/// An animation is stored on one row of a spritesheet.
pub struct AnimationData {
    pub total_frames: usize,
    /// How long to spend on one frame.
    pub frame_duration: usize,
}

#[derive(Clone)]
/// A spritesheet stores several animations in rows.
/// Each column is a new frame in each animation.
/// Every frame has the same height and width.
pub struct SpritesheetData {
    pub rows: Vec<AnimationData>,
    pub frame_width: usize,
    pub frame_height: usize,
}

/// AppearanceRecord stores which Appearances are currently loaded into the game
/// and ready to be used. Normally, they're inserted from the config::Server.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceRecord(pub Vec<String>);
impl AppearanceRecord {
    /// Creates an Appearance component with the given name.
    /// Panics if such an appearance can't be found.
    #[inline]
    pub fn appearance_of(&self, appearance: &str) -> Appearance {
        self.try_appearance_of(appearance)
            .unwrap_or_else(|e| panic!(e))
    }

    #[inline]
    pub fn try_appearance_of(&self, appearance: &str) -> Result<Appearance, String> {
        self.0
            .iter()
            .position(|r| appearance == r.as_str())
            .map(|index| Appearance(index))
            .ok_or_else(|| {
                format!(
                    concat!(
                        "Attempted to make an appearance from {:?},",
                        "but no such appearance found in AppearanceRecord.",
                        "Expected one of: {:?}",
                    ),
                    appearance, self.0,
                )
            })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
/// Something's index into the AppearanceRecord; which out of any of those possible
/// appearances they have.
///
/// Behavior can affect how something is rendered on the client, but
/// the appearance should never affect the behavior.
/// Therefore, this component isn't really used on the server all that much
/// except for when it needs to be sent down to the clients.
pub struct Appearance(usize);

#[cfg(feature = "flagged_appearances")]
impl Component for Appearance {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}
#[cfg(not(feature = "flagged_appearances"))]
impl Component for Appearance {
    type Storage = DenseVecStorage<Self>;
}

lazy_static::lazy_static! {
    pub static ref SPRITESHEETS: std::collections::HashMap<Appearance, SpritesheetData> = {
        //use Appearance::*;
        /*
        [
            (
                GleamyStalagmite,
                SpritesheetData {
                    rows: vec![AnimationData {
                        total_frames: 4,
                        frame_duration: 12,
                    }],
                    frame_width: 32,
                    frame_height: 32,
                },
            ),
            (
                Player,
                SpritesheetData {
                    rows: {
                        let mut rows = [
                            // (total frames, frame duration)
                            (7,     12),    // Casting
                            (8,     12),    // Jabbing
                            (9,     6),     // Walking
                            (6,     12),    // Swinging
                            (13,    12),    // Shooting
                        ]
                        .iter()
                        .fold(
                            Vec::new(),
                            // There are actually four rows for each of casting, jabbing etc.
                            |mut rows, &(total_frames, frame_duration)| {
                                for _ in 0..4 {
                                    rows.push(AnimationData {total_frames, frame_duration});
                                }
                                rows
                            },
                        );

                        // Dying
                        rows.push(AnimationData {
                            total_frames: 6,
                            frame_duration: 12,
                        });

                        rows
                    },
                    frame_width: 64,
                    frame_height: 64,
                },
            ),
        ]
        .iter()
        .cloned()
        .collect()*/
        std::collections::HashMap::new()
    };
}
