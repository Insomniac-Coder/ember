from generic_enum_mut_interface_imported_support.traits import Bump

pub enum Signal[T] implements Bump:
    Ready(value: T)

    fn bump(mut self) -> i32:
        return 42
