use glam::vec2;

use macroquad::*;

pub struct Slot {
    id: u64,
    item: Option<String>,
}
impl Slot {
    fn new(id: u64) -> Slot {
        Slot { id, item: None }
    }
}

pub enum FittingCommand {
    /// Remove item from this slot
    Unfit { target_slot: u64 },
    /// Fit item from inventory to slot
    Fit { target_slot: u64, item: String },
    /// Move item from one slot to another
    Refit { target_slot: u64, origin_slot: u64 },
}

pub struct Data {
    inventory: Vec<String>,
    item_dragging: bool,
    slots: Vec<(&'static str, Slot)>,
    fit_command: Option<FittingCommand>,
}
impl Data {
    pub fn new() -> Data {
        Data {
            inventory: vec![],
            item_dragging: false,
            slots: [
                "Left Mouse Button",
                "Right Mouse Button",
                "Middle Mouse Button",
                "Space",
                "\"1\"",
                "\"2\"",
                "\"3\"",
            ]
            .iter()
            .enumerate()
            .map(|(i, &s)| (s, Slot::new(i as u64)))
            .collect(),
            fit_command: None,
        }
    }

    /*
    fn slots(&mut self, ui: &mut Ui) {
        let item_dragging = &mut self.item_dragging;

        let fit_command = &mut self.fit_command;
        for (label, slot) in self.slots.iter_mut() {
            Group::new(hash!("grp", slot.id, &label), Vector2::new(210., 55.)).ui(ui, |ui| {
                let drag = Group::new(slot.id, Vector2::new(50., 50.))
                    // slot without item is not draggable
                    .draggable(slot.item.is_some())
                    // but could be a target of drag
                    .hoverable(*item_dragging)
                    // and is highlighted with other color when some item is dragging
                    .highlight(*item_dragging)
                    .ui(ui, |ui| {
                        if let Some(ref item) = slot.item {
                            ui.label(Vector2::new(5., 10.), &item);
                        }
                    });

                match drag {
                    // there is some item in this slot and it was dragged to another slot
                    Drag::Dropped(_, Some(id)) if slot.item.is_some() => {
                        *fit_command = Some(FittingCommand::Refit {
                            target_slot: id,
                            origin_slot: slot.id,
                        });
                    }
                    // there is some item in this slot and it was dragged out - unfit it
                    Drag::Dropped(_, None) if slot.item.is_some() => {
                        *fit_command = Some(FittingCommand::Unfit {
                            target_slot: slot.id,
                        });
                    }
                    // there is no item in this slot
                    // this is impossible - slots without items are non-draggable
                    Drag::Dropped(_, _) => unreachable!(),
                    Drag::Dragging(pos, id) => {
                        debug!("slots: pos: {:?}, id {:?}", pos, id);
                        *item_dragging = true;
                    }
                    Drag::No => {}
                }
                ui.label(Vector2::new(60., 20.), label);
            });
        }
    }

    fn inventory(&mut self, ui: &mut Ui) {
        let item_dragging = &mut self.item_dragging;
        for (n, item) in self.inventory.iter().enumerate() {
            let drag = Group::new(hash!("inventory", n), Vector2::new(50., 50.))
                .draggable(true)
                .ui(ui, |ui| {
                    ui.label(Vector2::new(5., 10.), &item);
                });

            match drag {
                Drag::Dropped(_, Some(id)) => {
                    self.fit_command = Some(FittingCommand::Fit {
                        target_slot: id,
                        item: item.clone(),
                    });
                    *item_dragging = false;
                }
                Drag::Dropped(_, _) => {
                    *item_dragging = false;
                }
                Drag::Dragging(pos, id) => {
                    debug!("inventory: pos: {:?}, id {:?}", pos, id);
                    *item_dragging = true;
                }
                _ => {}
            }
        }
    }*/

    fn set_item(&mut self, id: u64, item: Option<String>) {
        if let Some(slot) = self.slots.iter_mut().find(|(_, slot)| slot.id == id) {
            slot.1.item = item;
        }
    }
}

#[macroquad::main("egui showcase")]
async fn main() {
    let mut data = Data::new();

/*
    let mut data0 = String::new();
    let mut data1 = String::new();

    let mut text0 = String::new();
    let mut text1 = String::new();

    let mut number0 = 0.;
    let mut number1 = 0.;*/
    loop {
        clear_background(WHITE);

        use egui::{vec2, pos2, Window};

        draw_egui(|ui| {
            Window::new("Shop")
                .default_size(vec2(400.0, 200.0))
                .default_pos(pos2(320.0, 400.0))
                .show(ui.ctx(), |ui| {
                    for i in 0..30 {
                        ui.separator();
                        ui.label(&format!("Item N {}", i));
                        let mut col = ui.right_column(100.0);
                        col.label("10/10");
                        col.horizontal(|ui| {
                            ui.label("800 kr");
                            if ui.button("Buy").clicked {
                                data.inventory.push(format!("Item {}", i));
                            }
                        });
                        ui.allocate_space(vec2(0.0, 40.0));
                    }
                });

            Window::new("")
                .fixed_size(vec2(20.0, 20.0))
                .fixed_pos(pos2(50.0, 20.0))
                .collapsible(false)
                .show(ui.ctx(), |ui| drop(ui.label("Hello!")));
        });

        /*
        draw_window(
            hash!(),
            vec2(100., 220.),
            vec2(512., 420.),
            WindowParams {
                label: "Fitting window".to_string(),
                close_button: false,
                ..Default::default()
            },
            |ui| {
                Group::new(hash!(), Vector2::new(220., 400.)).ui(ui, |ui| {
                    data.slots(ui);
                });
                Group::new(hash!(), Vector2::new(280., 400.)).ui(ui, |ui| {
                    data.inventory(ui);
                });
            },
        );

        draw_window(
            hash!(),
            glam::vec2(470., 50.),
            glam::vec2(300., 300.),
            WindowParams {
                label: "Megaui Showcase Window".to_string(),
                ..Default::default()
            },
            |ui| {
                ui.tree_node(hash!(), "input", |ui| {
                    ui.label(None, "Some random text");
                    if ui.button(None, "click me") {
                        println!("hi");
                    }

                    ui.separator();

                    ui.label(None, "Some other random text");
                    if ui.button(None, "other button") {
                        println!("hi2");
                    }

                    ui.separator();

                    ui.input_field(hash!(), "<- input text 1", &mut data0);
                    ui.input_field(hash!(), "<- input text 2", &mut data1);
                    ui.label(
                        None,
                        &format!("Text entered: \"{}\" and \"{}\"", data0, data1),
                    );

                    ui.separator();
                });
                ui.tree_node(hash!(), "sliders", |ui| {
                    ui.slider(hash!(), "[-10 .. 10]", -10f32..10f32, &mut number0);
                    ui.slider(hash!(), "[0 .. 100]", 0f32..100f32, &mut number1);
                });
                ui.tree_node(hash!(), "editbox 1", |ui| {
                    ui.label(None, "This is editbox!");
                    ui.editbox(hash!(), megaui::Vector2::new(285., 165.), &mut text0);
                });
                ui.tree_node(hash!(), "editbox 2", |ui| {
                    ui.label(None, "This is editbox!");
                    ui.editbox(hash!(), megaui::Vector2::new(285., 165.), &mut text1);
                });
            },
        );*/

        match data.fit_command.take() {
            Some(FittingCommand::Unfit { target_slot }) => data.set_item(target_slot, None),
            Some(FittingCommand::Fit { target_slot, item }) => {
                data.set_item(target_slot, Some(item));
            }
            Some(FittingCommand::Refit {
                target_slot,
                origin_slot,
            }) => {
                let origin_item = data
                    .slots
                    .iter()
                    .find_map(|(_, slot)| {
                        if slot.id == origin_slot {
                            Some(slot.item.clone())
                        } else {
                            None
                        }
                    })
                    .flatten();
                data.set_item(target_slot, origin_item);
                data.set_item(origin_slot, None);
            }
            None => {}
        };

        next_frame().await;
    }
}
