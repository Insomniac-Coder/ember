// A32: a UI whose handlers get the UI itself (to count clicks, add more handlers).
pub struct Ui { pub handlers: Vec<Box<dyn FnMut(&mut Ui)>>, pub clicks: u32 }
impl Ui {
    pub fn fire(&mut self) {
        for h in self.handlers.iter_mut() {
            h(self);
        }
    }
}
