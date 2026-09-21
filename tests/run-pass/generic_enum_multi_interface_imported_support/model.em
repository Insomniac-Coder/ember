from generic_enum_multi_interface_imported_support.traits import Read
from generic_enum_multi_interface_imported_support.traits import Write

pub enum Signal[T] implements Read, Write:
    Ready(value: T)

    fn read(self) -> i32:
        return 20

    fn write(self) -> i32:
        return 22
