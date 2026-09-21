from generic_enum_generic_interface_default_imported_support.traits import Answer

pub enum Signal[T] implements Answer[T]:
    Ready(value: T)
