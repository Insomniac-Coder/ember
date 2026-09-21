from generic_enum_generic_inherited_defaults_imported_support.traits import Child
from generic_enum_generic_inherited_defaults_imported_support.traits import Parent

pub enum Signal[T] implements Parent[T], Child[T]:
    Ready(value: T)
