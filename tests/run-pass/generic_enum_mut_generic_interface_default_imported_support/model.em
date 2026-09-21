from generic_enum_mut_generic_interface_default_imported_support.traits import Bump

pub enum Signal[T] implements Bump[T]:
    Ready(value: T)
