from generic_enum_dynamic_box_imported_support.traits import Render

pub struct Token:
    pub value: i32

    fn drop(mut self):
        println(self.value)

pub enum Signal[T] implements Render:
    Ready(value: T)

    fn render(self) -> i32:
        return 42
