use glam::vec2;

use macroquad::{
    megaui::{widgets::Group, Drag, Ui, Vector2},
    *,
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

pub struct Data {
    inventory: Vec<String>,
    item_dragging: bool,
    slots: Vec<(&'static str, Slot)>,
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
        }
    }

    fn slots(&mut self, ui: &mut Ui) {
        let item_dragging = &mut self.item_dragging;

        for (label, item) in self.slots.iter_mut() {
            Group::new(hash!("grp", item.id, &label), Vector2::new(210., 55.)).ui(ui, |ui| {
                let drag = Group::new(item.id, Vector2::new(50., 50.))
                    .draggable(true)
                    .highlight(*item_dragging)
                    .ui(ui, |ui| {
                        if let Some(ref item) = item.item {
                            ui.label(Vector2::new(5., 10.), &item);
                        }
                    });

                match drag {
                    Drag::Dropped(_, id) => {
                        if id.map_or(true, |id| id != item.id) {
                            item.item = None;
                        }
                        *item_dragging = false;
                    }
                    Drag::Dragging => {
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
                    for slot in self.slots.iter_mut() {
                        if slot.1.id == id {
                            slot.1.item = Some(item.to_string());
                        }
                    }
                    *item_dragging = false;
                }
                Drag::Dropped { .. } => {
                    *item_dragging = false;
                }
                Drag::Dragging => {
                    *item_dragging = true;
                }
                _ => {}
            }
        }
    }
}

#[macroquad::main("UI showcase")]
async fn main() {
    let mut data = Data::new();

    let mut data0 = String::new();
    let mut data1 = String::new();

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
        );
        next_frame().await;
    }
}
