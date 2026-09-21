from generic_enum_interface_default_imported_support.traits import Answer

pub enum Signal[T] implements Answer:
    Ready(value: T)
