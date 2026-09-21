from generic_enum_generic_interface_imported_support.traits import Child
from generic_enum_generic_interface_imported_support.traits import Inspect

pub enum Signal[T] implements Inspect[T], Child[T]:
    Ready(value: T)

    fn inspect(self, value: T) -> i32:
        return 42

    fn child(self, value: T) -> i32:
        return 22
