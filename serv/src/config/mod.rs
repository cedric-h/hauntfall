//! There are two main avenues of configuring Hauntfall's Behavior.
//!
//! The first is raw data.
//! Examples of this include map.json and hauntfall_server_config.toml.
//!
//! There are also custom scripts that can be written to give individual levels
//! custom behavior and content.
//!
//! # Raw Data
//! These data formats are generally fairly straight forward, they store
//! simple lists of names, postitions, etc.
//!
//! This is a single config file that controls the behavior of the entire server.
//! It allows for the configuration of things like which map the game should use,
//! as well as which assets to load.
//! More documentation for hauntfall_server_config.toml can be found
//! on the ServerConfig struct.
//!
//! A map.json file is stored in each level, next to that level's script.
//! More documentation for map.json can be found on the MapEntry struct.
//!
//! # Scripting
//! Scripts can expose functions that are run in two different situations.
//! Once
//!
//! Scripts are able to change how the items stored in map.json manifest
//! themselves as they're being loaded. For example, using scripts you can
//! put items in chests, choose which of many possible enemy formations will
//! actually be present, make certain keys responsible for unlocking certain
//! doors and chests, and prevent certain parts of the map from spawning.
//!
//! Scripts are also able to conduct the flow of a map at runtime.
//! This exposes the widest breadth of custom behavior; through this avenue,
//! scripts are able to respond to events in the game like the death of a group
//! of enemies, perhaps by spawning a final boss or chests full of loot.
//! They can respond similarly to the player unlocking a door, or interacting with
//! a lever. They could create bridges that span chasms, or make all of the enemies
//! in a certain room no longer hostile. They could create traps which require skill
//! to navigate by having walls launch projectiles across hallways, forcing the player
//! to dash across.
use serde::Deserialize;

mod level;
pub use level::Level;

#[derive(Deserialize)]
/// Normally parsed in from `hauntfall.serverconfig`.
pub struct ServerConfig {
    pub appearance_record: comn::art::AppearanceRecord,
    pub level: String,
}
impl ServerConfig {
    pub fn parse() -> Self {
        let mut config = String::new();
        std::fs::File::open("./hauntfall_server_config.toml")
            .and_then(|mut f| {
                use std::io::Read;
                f.read_to_string(&mut config)
            })
            .expect("couldn't open hauntfall_server_config.toml");

        toml::from_str(&config).expect("hauntfall_server_config.toml file isn't proper TOML")
    }
}
