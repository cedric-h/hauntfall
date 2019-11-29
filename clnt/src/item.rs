use crate::prelude::*;
use comn::art::Appearance;
use comn::item::{Deposition, DropRequest, Inventory, SlotIndex};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stdweb::web::{document, html_element::ImageElement, IElement, INode, INonElementParentNode};

pub struct UpdateInventory {
    item_drop_events: Arc<Mutex<Vec<u32>>>,
    /// It's tricky for us to store entire SlotIndexes next to HTML5 elements,
    /// so we just use this little HashMap.
    ents_to_slot: HashMap<u32, SlotIndex>,
}
impl Default for UpdateInventory {
    fn default() -> Self {
        let item_drop_events = Arc::new(Mutex::new(Vec::new()));

        {
            let item_drop_events = item_drop_events.clone();
            let drop_item = move |id: u64| {
                item_drop_events
                    .lock()
                    .expect("couldn't lock item drop events")
                    .push(id as u32);
            };
            js! {
                let drop_item = @{drop_item};
                $("body").droppable({
                    accept: ".item",
                    drop: function(e, o) {
                        //width: 400px;
                        //height: 225px;
                        if (
                            (o.position.top < -30 || o.position.top > 225 + 30) ||
                            (o.position.left < -30 || o.position.left > 400 + 30)
                        ) {
                            drop_item(Math.floor(o.draggable[0].id));
                        }
                    }
                });
            }
        }

        Self {
            item_drop_events,
            ents_to_slot: HashMap::new(),
        }
    }
}
impl<'a> System<'a> for UpdateInventory {
    type SystemData = (
        Entities<'a>,
        Read<'a, crate::net::ServerConnection>,
        Read<'a, crate::net::ServerToLocalIds>,
        WriteStorage<'a, Inventory>,
        ReadStorage<'a, Appearance>,
    );

    fn run(
        &mut self,
        (ents, sc, server_to_local_ids, mut inventories, appearances): Self::SystemData,
    ) {
        if let Ok(mut item_drops) = self.item_drop_events.lock() {
            for ent_id in item_drops.drain(..) {
                sc.insert_comp(DropRequest {
                    item_index: self.ents_to_slot[&ent_id].clone(),
                });
            }
        }

        for (ent, inventory) in (&*ents, inventories.drain()).join() {
            let player_id = ent.id().to_string();

            // either the inventory div exists,
            let inventory_div = match document().get_element_by_id(&player_id) {
                // meaning that it needs to be grabbed and cleared before use,
                Some(div) => {
                    for _ in 0..div.child_nodes().len() {
                        div.remove_child(&div.first_child().unwrap()).unwrap();
                    }
                    div
                }
                // or it doesn't exist, so it needs to be made.
                None => {
                    let div = document().create_element("div").unwrap();

                    div.class_list().add("box").unwrap();
                    div.class_list().add("inventory").unwrap();
                    div.set_attribute("id", &player_id).unwrap();

                    document().body().unwrap().append_child(&div);

                    js!($("#" + @{player_id}).draggable());
                    div
                }
            };

            for (index, slot) in inventory.reserved().chain(inventory.loose()) {
                // two ways to get an image,
                let image = slot
                    // be a non-empty slot and have an appearance,
                    .map(|item_server_id| {
                        let item_ent = ents.entity(
                            *server_to_local_ids
                                .0
                                .get_by_left(&item_server_id)
                                .expect("can't render item; invalid server id"),
                        );
                        let appearance = appearances
                            .get(item_ent)
                            .expect("inventory item has no appearance");

                        format!("./img/{:?}.png", appearance)
                    })
                    // or be a Reserved Slot that has a fancy image.
                    .or_else(|| {
                        if let SlotIndex::Reserved(item) = index {
                            Some(format!("./img/{:?}Slot.png", item))
                        } else {
                            None
                        }
                    });

                let slot_div = match image {
                    Some(image_path) => {
                        // set image up to load
                        let new_img = ImageElement::with_size(64, 64);
                        new_img.class_list().add("item").unwrap();
                        new_img.set_src(&image_path);
                        new_img.set_alt(&image_path);

                        let slot_div = document().create_element("div").unwrap();
                        slot_div.class_list().add("item_wrapper").unwrap();
                        slot_div.append_child(&new_img);

                        // this + the dragging that's added in a bit
                        // makes actual item slots do their thing,
                        // whereas empty ones don't do much.
                        if let Some(item) = slot {
                            self.ents_to_slot.insert(*item, index.clone());

                            new_img.set_attribute("id", &item.to_string()).unwrap();
                        }

                        slot_div
                    }
                    // it doesn't have an image, so just make a square
                    None => {
                        let slot_div = document().create_element("div").unwrap();
                        slot_div.class_list().add("item_wrapper").unwrap();
                        slot_div
                    }
                };

                if let SlotIndex::Loose(row, col) = index {
                    slot_div
                        .set_attribute(
                            "style",
                            &format!(
                                "position:absolute; left:{}px; top:{}px;",
                                8 + row * 76,
                                25 + (col + 1) * 76,
                            ),
                        )
                        .unwrap();
                }

                inventory_div.append_child(&slot_div);

                // Now that the slot div is in the document,
                // we can make it draggable.
                // (ofc, we only want to do that if it's an item)
                if let Some(item) = slot {
                    js! {
                        $("#" + @{item}).draggable({
                            revert: true
                        });
                    }
                }
            }

            inventory_div.append_child(&document().create_element("hr").unwrap())
        }
    }
}

/// This system removes the Pos component from entities
/// with the comn::item::Deposition Component, making the entities
/// unable to exist physically, effectively turning them into items.
///
/// On the client, because of three.js ruining everything, we have to
/// also remove the Appearance component as well to make them disappear.
pub struct DepositionItems;
impl<'a> System<'a> for DepositionItems {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Deposition>,
        WriteStorage<'a, Appearance>,
        WriteStorage<'a, Pos>,
    );

    fn run(&mut self, (ents, mut deposes, mut appearances, mut poses): Self::SystemData) {
        for (ent, _) in (&*ents, deposes.drain()).join() {
            appearances
                .remove(ent)
                .expect("Couldn't deposition an entity by removing it's appearance.");
            poses
                .remove(ent)
                .expect("Couldn't literally deposition an entity");
        }
    }
}
