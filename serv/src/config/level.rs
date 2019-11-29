use comn::prelude::*;
use forge::Engine;
use log::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::{fs::File, io::Read};

fn f32_list_from_forge(forge_val: &Vec<forge::Value>) -> Result<Vec<f32>, &forge::Value> {
    forge_val.iter().fold(Ok(Vec::new()), |acc, x| {
        if let Ok(mut acc) = acc {
            match x {
                forge::Value::Number(f) => {
                    acc.push(*f as f32);
                    Ok(acc)
                }
                _ => Err(x),
            }
        } else {
            acc
        }
    })
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
        let script_path = format!("./levels/{}/script.rhai", level);

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

        // open the script.rhai file for the level
        let mut script = String::new();
        File::open(&script_path)
            .and_then(|mut f| f.read_to_string(&mut script))
            .unwrap_or_else(|e| {
                panic!(
                    concat!(
                        "couldn't read the script.rhai file at {}: {} ",
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

    pub fn load_map(&mut self, world: &mut comn::specs::World, config: &super::ServerConfig) {
        use comn::specs::{Builder, WorldExt};
        use comn::Hitbox;
        use forge::Value as v;

        let processed_map = self
            .engine
            .eval("load_map(map_entries)")
            .unwrap_or_else(|e| {
                panic!(
                    "Level Script Error: level {}'s load_map function failed: {}",
                    self.level, e
                )
            });

        if let v::List(l) = processed_map {
            for ent_map in l.borrow().iter() {
                if let v::Map(m) = ent_map {
                    let ent_map: HashMap<String, v> = m
                        .borrow()
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k.get_display_text().unwrap(), v))
                        .collect();

                    let mut builder = world.create_entity();

                    if let Some(v::List(l)) = ent_map.get("location") {
                        let loc = f32_list_from_forge(&l.borrow()).unwrap_or_else(|e| {
                            panic!(
                                "Invalid data type in location list for {}, found: \"{}\"",
                                ent_map.get("name").unwrap(),
                                e
                            )
                        });
                        builder = builder.with(Pos::vec(Vec2::new(loc[0], loc[1])))
                    } else {
                        error!("no location on entity {}", ent_map.get("name").unwrap())
                    }
                    if let Some(v::String(a)) = ent_map.get("appearance") {
                        let appearance_name = a.borrow();
                        builder = builder.with(
                            config
                                .appearance_record
                                .try_appearance_of(&appearance_name)
                                .unwrap_or_else(|e| {
                                    panic!("Invalid appearance name when parsing map: {}", e)
                                }),
                        );
                    } else {
                        error!("no appearance on entity {}", ent_map.get("name").unwrap())
                    }
                    if let Some(v::List(l)) = ent_map.get("hitbox_dimensions") {
                        let d = f32_list_from_forge(&l.borrow()).unwrap_or_else(|e| {
                            panic!(
                                "Invalid data type in hitbox_dimensions list for {}, found: \"{}\"",
                                ent_map.get("name").unwrap(),
                                e
                            )
                        });
                        builder = builder.with(Hitbox::vec(Vec2::new(d[0], d[1]) / 2.0));
                    }
                    builder.build();
                }
            }
        }
    }
}
