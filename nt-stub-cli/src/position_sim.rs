use eframe::egui::{self, Pos2, Scene};
use egui::ViewportBuilder;
use lagan::{client::Client, entry::Entry, Instance};
pub fn run() {
    let client = Client::builder().address("127.0.0.1:5810".parse().unwrap()).build();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Pos Sim",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(App {
                position: Pos2 { x: 200.0, y: 100.0 },
                scene_view_rect: egui::Rect::ZERO,
                movement_keys: MovementKeys {
                    left: false,
                    right: false,
                    up: false,
                    down: false,
                },
                client: client.entry("/robot_pos"),
            }))
        }),
    )
    .unwrap();
}

#[derive(Debug)]
struct MovementKeys {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

struct App<'a> {
    position: Pos2,
    scene_view_rect: egui::Rect,
    movement_keys: MovementKeys,
    client: Entry<'a, Client>,
}
impl<'a> eframe::App for App<'a> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            Scene::new()
                .zoom_range(0.1..=2.0)
                .show(ui, &mut self.scene_view_rect, |ui| {
                    ui.image(egui::include_image!("./reefscape.png"));
                    ui.painter().circle_filled(
                        self.position,
                        18.0,
                        egui::Color32::from_rgb(0xFF, 0x36, 0x36),
                    );

                    ui.input(|state| {
                        if state.key_pressed(egui::Key::ArrowRight) {
                            self.movement_keys.right = true;
                        }
                        if state.key_pressed(egui::Key::ArrowLeft) {
                            self.movement_keys.left = true;
                        }
                        if state.key_pressed(egui::Key::ArrowUp) {
                            self.movement_keys.up = true;
                        }
                        if state.key_pressed(egui::Key::ArrowDown) {
                            self.movement_keys.down = true;
                        }

                        if state.key_released(egui::Key::ArrowRight) {
                            self.movement_keys.right = false;
                        }
                        if state.key_released(egui::Key::ArrowLeft) {
                            self.movement_keys.left = false;
                        }
                        if state.key_released(egui::Key::ArrowUp) {
                            self.movement_keys.up = false;
                        }
                        if state.key_released(egui::Key::ArrowDown) {
                            self.movement_keys.down = false;
                        }
                    });

                    if self.movement_keys.right {
                        self.position.x += 1.0;
                    }
                    if self.movement_keys.left {
                        self.position.x -= 1.0;
                    }
                    if self.movement_keys.up {
                        self.position.y -= 1.0;
                    }
                    if self.movement_keys.down {
                        self.position.y += 1.0;
                    }
                    if self.movement_keys.down
                        || self.movement_keys.up
                        || self.movement_keys.left
                        || self.movement_keys.right
                    {
                        ctx.request_repaint();
                        _ = self.client.set_value_f64_array(vec![self.position.x as _, self.position.y as _, 0.0]);
                    }
                })
        });
    }
}
