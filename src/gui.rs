use direct_gui::*;
use direct_gui::controls::*;
use blit::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GuiEvent {
    None,
    BuyArcherButton
}

pub struct IngameGui {
    gui: Gui,
    cs: ControlState,
    size: (i32, i32),
    bg_pos: (i32, i32),

    menu_bg: BlitBuffer,
    archer_button: ControlRef
}

impl IngameGui {
    pub fn new(size: (i32, i32)) -> Self {
        // Setup the GUI system
        let mut gui = Gui::new(size);

        let menu_bg = BlitBuffer::from_memory(include_bytes!("../resources/gui/iconbar.png.blit")).unwrap();

        let bg_x = (size.0 - menu_bg.size().0) / 2;
        let bg_y = size.1 - menu_bg.size().1;

        let archer_button_img = gui.load_sprite_from_memory(include_bytes!("../resources/gui/archer-button.png.blit")).unwrap();
        let archer_button = gui.register(Button::new_with_sprite(archer_button_img).with_pos(bg_x + 8, bg_y + 12));
        
        IngameGui {
            gui, size, menu_bg, archer_button,

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

        {
            let archer_button: &Button<Image> = self.gui.get(self.archer_button).unwrap();
            if !self.cs.mouse_down && archer_button.pressed() {
                result = GuiEvent::BuyArcherButton;
            }
        }

        self.gui.update(&self.cs);

        result
    }

    pub fn render(&mut self, buffer: &mut Vec<u32>) {
        self.menu_bg.blit(buffer, self.size.0 as usize, self.bg_pos);

        self.gui.draw_to_buffer(buffer);
    }
}
