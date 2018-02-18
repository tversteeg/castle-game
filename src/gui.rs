use direct_gui::*;
use direct_gui::controls::*;
use blit::*;

pub struct IngameGui {
    gui: Gui,
    cs: ControlState,
    size: (i32, i32),

    menu_bg: BlitBuffer
}

impl IngameGui {
    pub fn new(size: (i32, i32)) -> Self {
        // Setup the GUI system
        let gui = Gui::new(size);
        
        IngameGui {
            gui, size,
            cs: ControlState::default(),

            menu_bg: BlitBuffer::from_memory(include_bytes!("../resources/gui/iconbar.png.blit")).unwrap()
        }
    }

    pub fn handle_mouse(&mut self, pos: (i32, i32), left_is_down: bool) {
        self.cs.mouse_pos = pos;
        self.cs.mouse_down = left_is_down;
    }

    pub fn render(&mut self, buffer: &mut Vec<u32>) {
        self.gui.update(&self.cs);

        let bg_x = (self.size.0 - self.menu_bg.size().0) / 2;
        let bg_y = self.size.1 - self.menu_bg.size().1;
        self.menu_bg.blit(buffer, self.size.0 as usize, (bg_x, bg_y));

        self.gui.draw_to_buffer(buffer);
    }
}
