use comn::specs::{Builder, World, WorldExt};
use forge::Engine;
// std
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::{fs::File, io::Read};
// us
use super::ServerConfig;
use comn::prelude::*;

type StringForgeMap = HashMap<String, forge::Value>;

fn vec3_from_forge_list(forge_vals: &Vec<forge::Value>) -> Result<na::Vector3<f32>, &forge::Value> {
    let mut vec: na::Vector3<f32> = na::zero();

    for (index, val) in forge_vals.iter().enumerate() {
        match val {
            forge::Value::Number(f) => {
                vec[index] = *f as f32;
            }
            _ => return Err(val),
        }
    }

    Ok(vec)
}

fn string_forge_map_from(forge_map: &forge::Value) -> Result<StringForgeMap, ()> {
    if let forge::Value::Map(map_ref) = forge_map {
        Ok(map_ref
            .borrow()
            .clone()
            .into_iter()
            .map(|(k, v)| (k.get_display_text().unwrap(), v))
            .collect())
    } else {
        Err(())
    }
}

fn json_to_forge(json: serde_json::Value) -> forge::Value {
    use forge::Value as fg;
    use serde_json::Value as js;
    match json {
        js::Bool(b) => fg::Boolean(b),
        js::String(s) => fg::String(Rc::new(RefCell::new(s))),
        js::Array(v) => fg::List(Rc::new(RefCell::new(
            v.into_iter().map(|x| json_to_forge(x)).collect(),
        ))),
        js::Number(n) => fg::Number(n.as_f64().expect("number from JSON couldn't f64")),
        js::Null => fg::Null,
        js::Object(_) => unimplemented!(),
    }
}

pub struct Level {
    engine: Engine,
    level: String,
}

impl Level {
    pub fn from_name(level: String) -> Self {
        let map_path = format!("./levels/{}/map.json", level);
        let script_path = format!("./levels/{}/script.fg", level);

        // parse the map
        let map_entries_json: Vec<HashMap<String, serde_json::Value>> =
            serde_json::from_reader(File::open(&map_path).expect("couldn't open map.json"))
                .expect("map file isn't proper JSON");
        let map_entries_forge = forge::Value::List(Rc::new(RefCell::new(
            map_entries_json
                .into_iter()
                .map(|entry| {
                    forge::Value::Map(Rc::new(RefCell::new(
                        entry
                            .into_iter()
                            .map(|(name, json_value)| {
                                (
                                    forge::Value::String(Rc::new(RefCell::new(name))),
                                    json_to_forge(json_value),
                                )
                            })
                            .collect(),
                    )))
                })
                .collect(),
        )));

        // open the script.fg file for the level
        let mut script = String::new();
        File::open(&script_path)
            .and_then(|mut f| f.read_to_string(&mut script))
            .unwrap_or_else(|e| {
                panic!(
                    concat!(
                        "couldn't read the script.fg file at {}: {} ",
                        "Perhaps an invalid/nonexistent level name was ",
                        "provided in the hauntfall_server_config.toml?",
                    ),
                    script_path, e
                )
            });

        // validate the script, prepare to run its functions
        let mut engine = Engine::build()
            .with_global("map_entries", map_entries_forge)
            .finish();

        engine
            .exec(&script)
            .unwrap_or_else(|e| panic!("Couldn't parse level script: {}", e));

        Self { engine, level }
    }

    pub fn load_map(&mut self, world: &mut World, config: &ServerConfig) -> Result<(), String> {
        let processed_map = self
            .engine
            .eval("load_map(map_entries)")
            .unwrap_or_else(|e| {
                panic!(
                    "Level Script Error: level {}'s load_map function failed: {}",
                    self.level, e
                )
            });

        if let forge::Value::List(l) = processed_map {
            for ent_map in l.borrow().iter() {
                if let Ok(ent_map) = string_forge_map_from(ent_map) {
                    // save the ent_map's name in case of an error
                    let name = ent_map.get("name").unwrap().clone();

                    Self::process_map_entry(ent_map, world, config).map_err(|e| {
                        format!(
                            concat!(
                                "Couldn't make entity from map entry \"{}\" returned from ",
                                "level {}'s script's load_map function entry: {}",
                            ),
                            name, self.level, e
                        )
                    })?;
                }
            }
        }

        Ok(())
    }

    fn process_map_entry(
        map: StringForgeMap,
        world: &mut World,
        config: &ServerConfig,
    ) -> Result<(), String> {
        use comn::Hitbox;
        use forge::Value as v;

        let mut builder = world.create_entity();

        // find location in map, add position component to entity.
        // if no location, you just don't get a location that's fine
        if let Some(v::List(l)) = map.get("location") {
            let loc = vec3_from_forge_list(&l.borrow())
                .map_err(|e| format!("Invalid data type in location list, found: \"{}\"", e))?;
            builder = builder.with(Pos::vec(loc.xy()))
        }

        // find an appearance name in map, turn it into an appearance
        // component using the config, and add that to the entity.
        // if no appearance, you just don't get an appearance that's fine
        if let Some(v::String(a)) = map.get("appearance") {
            let appearance_name = a.borrow();
            builder = builder.with(
                config
                    .appearance_record
                    .try_appearance_of(&appearance_name)
                    .map_err(|e| format!("Invalid appearance name when parsing map: {}", e))?,
            );
        }

        // see if hitbox dimensions can be found in the map; if they can,
        // then this entity needs a hitbox.
        if let Some(v::List(l)) = map.get("hitbox_dimensions") {
            let dim = vec3_from_forge_list(&l.borrow()).map_err(|e| {
                format!(
                    "Invalid data type in hitbox_dimensions list found: \"{}\"",
                    e
                )
            })?;
            builder = builder.with(Hitbox::vec(dim.xy() / 2.0));
        }

        // see if chaser data can be found in the map; if it can,
        // then this entity needs that component.
        if let Some(Ok(chaser_map)) = map.get("chaser").map(|x| string_forge_map_from(x)) {
            use crate::combat::Alignment;
            let target: Alignment = serde_json::from_str(&format!(
                "\"{}\"",
                chaser_map
                    .get("target")
                    .ok_or_else(|| "chaser present but no target provided".to_string())?,
            ))
            .map_err(|_| {
                concat!(
                    "invalid alignment provided in chaser, ",
                    "must be one of 'Players', 'Enemies', 'All', 'Neither'"
                )
            })?;

            let distance = match chaser_map
                .get("distance")
                .ok_or_else(|| "no distance provided for chaser".to_string())?
            {
                v::Number(f) => *f as f32,
                _ => return Err("chaser distance isn't a number".to_string()),
            };

            let speed = match chaser_map
                .get("speed")
                .ok_or_else(|| "no speed provided for chaser".to_string())?
            {
                v::Number(f) => *f as f32,
                _ => return Err("chaser speed isn't a number".to_string()),
            };

            builder = builder.with(
                crate::combat::ChaserBuilder {
                    target,
                    distance,
                    speed,
                }
                .build(),
            );
        }

        // finally build that entity
        builder.build();
        Ok(())
    }
}
