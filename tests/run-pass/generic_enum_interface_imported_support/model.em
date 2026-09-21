from generic_enum_interface_imported_support.traits import Render

pub enum Signal[T] implements Render:
    Value(value: T)

    fn render(self) -> i32:
        return 42
