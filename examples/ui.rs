
use macroquad::{
    megaui::{widgets::Group, Drag, Ui, Vector2},
    prelude::*,
};

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
            slots: vec![
                ("Left Mouse Button", Slot::new(hash!())),
                ("Right Mouse Button", Slot::new(hash!())),
                ("Middle Mouse Button", Slot::new(hash!())),
                ("Space", Slot::new(hash!())),
                ("\"1\"", Slot::new(hash!())),
                ("\"2\"", Slot::new(hash!())),
                ("\"3\"", Slot::new(hash!())),
            ],
            fit_command: None,
        }
    }

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
    }

    fn set_item(&mut self, id: u64, item: Option<String>) {
        if let Some(slot) = self
            .slots
            .iter_mut()
            .find(|(_, slot)| slot.id == id)
        {
            slot.1.item = item;
        }
    }
}

#[macroquad::main("UI showcase")]
async fn main() {
    let mut data = Data::new();

    let mut string = String::new();
    let mut float = 1337.0;

    let mut text0 = String::new();
    let mut text1 = String::new();

    let mut number0 = 0.;
    let mut number1 = 0.;
    loop {
        clear_background(WHITE);

        draw_window(
            hash!(),
            vec2(400., 200.),
            vec2(320., 400.),
            WindowParams {
                label: "Shop".to_string(),
                close_button: false,
                ..Default::default()
            },
            |ui| {
                for i in 0..30 {
                    Group::new(hash!("shop", i), Vector2::new(300., 80.)).ui(ui, |ui| {
                        ui.label(Vector2::new(10., 10.), &format!("Item N {}", i));
                        ui.label(Vector2::new(260., 40.), "10/10");
                        ui.label(Vector2::new(200., 58.), &format!("{} kr", 800));
                        if ui.button(Vector2::new(260., 55.), "buy") {
                            data.inventory.push(format!("Item {}", i));
                        }
                    });
                }
            },
        );

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

                    ui.input_string(hash!(), "<- input text", &mut string);
                    ui.input_float(hash!(), "<- input float", &mut float);
                    ui.label(
                        None,
                        &format!("Text entered: \"{}\" and \"{}\"", string, float),
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
        );

        draw_window(
            hash!(),
            glam::vec2(10., 10.),
            glam::vec2(50., 20.),
            WindowParams {
                titlebar: false,
                ..Default::default()
            },
            |ui| {
                ui.label(None, "Hello!");
            },
        );

        match data.fit_command.take() {
            Some(FittingCommand::Unfit { target_slot }) =>                 data.set_item(target_slot, None),
            Some(FittingCommand::Fit { target_slot, item }) => {
                data.set_item(target_slot, Some(item));
            }
            Some(FittingCommand::Refit {
                target_slot,
                origin_slot,
            }) => {
                let origin_item = data.slots
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
            },
            None => {}
        };

        next_frame().await;
    }
}
