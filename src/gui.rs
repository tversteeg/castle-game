use specs::*;
use direct_gui::*;
use direct_gui::controls::*;
use blit::*;

use super::*;

#[derive(Component, Debug)]
pub struct FloatingText {
    pub text: String,
    pub pos: Point,
    pub time_alive: f64
}

pub struct FloatingTextSystem;
impl<'a> System<'a> for FloatingTextSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       WriteStorage<'a, FloatingText>);

    fn run(&mut self, (entities, dt, mut text): Self::SystemData) {
        let dt = dt.to_seconds();

        for (entity, text) in (&*entities, &mut text).join() {
            // Kill the text if it's time alive is up
            text.time_alive -= dt;
            if text.time_alive <= 0.0 {
                let _ = entities.delete(entity);
                continue;
            }

            // Float the text up
            text.pos.0.y -= dt * 20.0;
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GuiEvent {
    None,
    BuyArcherButton,
    BuySoldierButton
}

pub struct IngameGui {
    gui: Gui,
    cs: ControlState,
    size: (i32, i32),
    bg_pos: (i32, i32),

    menu_bg: BlitBuffer,
    archer_button: ControlRef,
    soldier_button: ControlRef
}

impl IngameGui {
    pub fn new(size: (i32, i32)) -> Self {
        // Setup the GUI system
        let mut gui = Gui::new(size);

        let menu_bg = BlitBuffer::from_memory(include_bytes!("../resources/gui/iconbar.blit")).unwrap();

        let bg_x = (size.0 - menu_bg.size().0) / 2;
        let bg_y = size.1 - menu_bg.size().1;

        let archer_button_img = gui.load_sprite_from_memory(include_bytes!("../resources/gui/archer-button.blit")).unwrap();
        let archer_button = gui.register(Button::new_with_sprite(archer_button_img).with_pos(bg_x + 8, bg_y + 12));
        
        let soldier_button_img = gui.load_sprite_from_memory(include_bytes!("../resources/gui/soldier-button.blit")).unwrap();
        let soldier_button = gui.register(Button::new_with_sprite(soldier_button_img).with_pos(bg_x + 40, bg_y + 12));

        IngameGui {
            gui, size, menu_bg, archer_button, soldier_button,

            cs: ControlState::default(),
            bg_pos: (bg_x, bg_y)
        }
    }

    pub fn handle_mouse(&mut self, pos: (i32, i32), left_is_down: bool) {
        self.cs.mouse_pos = pos;
        self.cs.mouse_down = left_is_down;
    }

    pub fn update(&mut self) -> GuiEvent {
        let mut result = GuiEvent::None;

        // Set the state to the buttons pressed
        {
            // If the mouse is not down anymore but the button state is still pressed means that
            // the mouse was just released
            let archer_button: &Button<Image> = self.gui.get(self.archer_button).unwrap();
            if !self.cs.mouse_down && archer_button.pressed() {
                result = GuiEvent::BuyArcherButton;
            }

            let soldier_button: &Button<Image> = self.gui.get(self.soldier_button).unwrap();
            if !self.cs.mouse_down && soldier_button.pressed() {
                result = GuiEvent::BuySoldierButton;
            }
        }

        self.gui.update(&self.cs);

        result
    }

    pub fn draw_label(&mut self, buffer: &mut Vec<u32>, text: &String, pos: (i32, i32)) {
        let default_font = self.gui.default_font();
        self.gui.draw_label(buffer, default_font, text, pos);
    }

    pub fn render(&mut self, buffer: &mut Vec<u32>) {
        self.menu_bg.blit(buffer, self.size.0 as usize, self.bg_pos);

        self.gui.draw_to_buffer(buffer);
    }
}
