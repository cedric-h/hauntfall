use comn::specs::{LazyUpdate, World, WorldExt};
use specs::{Component, Entity};
// std
use std::fmt::Debug;
use std::marker::PhantomData;
use std::{fs::File, io::Read};
// us
use super::ServerConfig;
use crate::combat::Chaser;
use comn::prelude::*;
// script
use pyo3::prelude::*;
use pyo3::PyTypeInfo;

trait ComponentFactory {
    fn try_py_insert<'p>(
        &self,
        py: Python<'p>,
        obj: &PyObject,
        lu: &LazyUpdate,
        e: Entity,
    ) -> Result<(), ()>;
}

struct ComponentEntry<C: PyTypeInfo + Debug + Component + Clone + Send + Sync> {
    pd: PhantomData<C>,
}
impl<C: PyTypeInfo + Debug + Component + Clone + Send + Sync> ComponentEntry<C> {
    const INSTANCE: Self = Self { pd: PhantomData };
}
impl<C: PyTypeInfo + Component + Debug + Clone + Send + Sync> ComponentFactory
    for ComponentEntry<C>
{
    fn try_py_insert<'p>(
        &self,
        py: Python<'p>,
        obj: &PyObject,
        lu: &LazyUpdate,
        e: Entity,
    ) -> Result<(), ()> {
        if let Ok(c) = obj.cast_as::<C>(py) {
            log::info!("{:?}", c);
            lu.insert(e, c.clone());
            Ok(())
        } else {
            Err(())
        }
    }
}

struct ComponentFactoryRegistry(Vec<&'static dyn ComponentFactory>);
impl ComponentFactoryRegistry {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn register<C: PyTypeInfo + Component + Debug + Clone + Send + Sync>(&mut self) {
        self.0.push(&ComponentEntry::<C>::INSTANCE);
    }

    pub fn try_py_insert<'p>(
        &self,
        py: Python<'p>,
        obj: PyObject,
        lu: &LazyUpdate,
        a: Entity,
    ) -> Result<(), String> {
        for cf in self.0.iter() {
            if let Ok(()) = cf.try_py_insert(py, &obj, lu, a) {
                return Ok(());
            }
        }
        Err("No such component!".to_string())
    }
}

pub struct Level {
    level: String,
    registry: ComponentFactoryRegistry,
}

impl Level {
    pub fn from_name(level: String) -> Self {
        // open the script.py file for the level
        let script_path = format!("./levels/{}/script.py", level);
        let mut script_src = String::new();
        File::open(&script_path)
            .and_then(|mut f| f.read_to_string(&mut script_src))
            .unwrap_or_else(|e| {
                panic!(
                    concat!(
                        "couldn't read the script.py file at {}: {} ",
                        "Perhaps an invalid/nonexistent level name was ",
                        "provided in the hauntfall_server_config.toml?",
                    ),
                    script_path, e
                )
            });

        // interpret the Python file
        let gil = Python::acquire_gil();
        let py = gil.python();

        let script = PyModule::from_code(py, &script_src, &format!("{}.py", &level), "level")
            .unwrap_or_else(|e| {
                e.print(py);
                panic!("Couldn't load Python script for level {}", &level)
            });
        script
            .add("level_name", level.clone())
            .expect("Couldn't insert level name into module.");

        // register all Rust types
        let mut registry = ComponentFactoryRegistry::new();

        macro_rules! register_components {
            ( $( $t:tt , )* ) => {
                $(
                    registry.register::<$t>();
                    script.add_class::<$t>()
                        .expect(concat!(
                            "couldn't add class ",
                            stringify!($t)
                        ));
                )*
            }
        }

        use comn::art::{Appearance, AppearanceRecord};

        script.add_class::<AppearanceRecord>().unwrap();

        #[rustfmt::skip]
        register_components!(
            Chaser,
            Appearance,
        );

        Self { level, registry }
    }

    #[inline]
    pub fn load_map(&mut self, world: &mut World, config: &ServerConfig) -> Result<(), String> {
        // parse the map
        let map_path = format!("./levels/{}/map.json", self.level);
        let mut map_json = String::new();
        File::open(&map_path)
            .and_then(|mut f| f.read_to_string(&mut map_json))
            .map_err(|e| format!("couldn't read the map.json file at {}: {} ", map_path, e))?;

        // Run the python function
        let gil = Python::acquire_gil();
        let py = gil.python();

        // grab the level module
        let script = PyModule::import(py, "level").expect("Couldn't get level module!");

        // share some of the config with it.
        script
            .add("appearance_record", config.appearance_record.clone())
            .expect("Couldn't insert the appearance_record into the level module!");

        // prepare to create the entities the script tells us about
        let ents = world.entities();
        let lu = world.read_resource::<LazyUpdate>();

        // run the script and collect the list of entities (which are lists of components) from it.
        let output: Vec<Vec<PyObject>> = script
            .call1("load_map", (map_json,))
            .map_err(|e| {
                e.print(py);
                format!("{}.py's load_map function failed.", self.level)
            })?
            .extract::<Vec<Vec<PyObject>>>()
            .map_err(|e| {
                e.print(py);
                format!(
                    concat!(
                        "{}.py's load_map function gave output in the wrong format, ",
                        "expected a list of list of components."
                    ),
                    self.level
                )
            })?;

        // loop over each entity's list of components
        for entity_components in output {
            // make an index for that entity
            let ent = ents.create();

            // add each of the components to it.
            for comp in entity_components {
                self.registry
                    .try_py_insert(py, comp, &lu, ent)
                    .map_err(|e| {
                        format!(
                            "{}.py's load_map function gave invalid output: {}",
                            self.level, e
                        )
                    })?;
            }
        }

        Ok(())
    }
}
