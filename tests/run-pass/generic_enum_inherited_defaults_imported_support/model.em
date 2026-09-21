from generic_enum_inherited_defaults_imported_support.traits import Child
from generic_enum_inherited_defaults_imported_support.traits import Parent

pub enum Signal[T] implements Parent, Child:
    Ready(value: T)
