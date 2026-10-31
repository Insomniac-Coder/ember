from generic_enum_inherited_interface_imported_support.traits import Child
from generic_enum_inherited_interface_imported_support.traits import Parent

pub enum Signal[T] implements Parent, Child:
    Ready(value: T)

    fn parent() -> i32:
        return 20

    fn child() -> i32:
        return 22
