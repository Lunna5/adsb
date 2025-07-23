pub mod map;

pub trait Component {
    fn draw(&mut self, ui: &egui::Context);
}

pub trait Window: Component {
    fn open(&mut self);
    fn close(&mut self);
    fn is_open(&self) -> bool;
}