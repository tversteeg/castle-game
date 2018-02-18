use direct_gui::*;
use direct_gui::controls::*;

pub struct IngameGui {
    gui: Gui,
    cs: ControlState
}

impl IngameGui {
    pub fn new(size: (i32, i32)) -> Self {
        // Setup the GUI system
        let gui = Gui::new(size);
        
        IngameGui {
            gui,
            cs: ControlState::default()
        }
    }

    pub fn handle_mouse(&mut self, pos: (i32, i32), left_is_down: bool) {
        self.cs.mouse_pos = pos;
        self.cs.mouse_down = left_is_down;
    }

    pub fn render(&mut self, buffer: &mut Vec<u32>) {
        self.gui.update(&self.cs);
        self.gui.draw_to_buffer(buffer);
    }
}
