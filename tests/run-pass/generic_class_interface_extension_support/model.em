from generic_class_interface_extension_support.traits import Measure

pub class Holder[T]:
    value: i32
    marker: T

extend[T] Holder[T] implements Measure:
    fn measure(self) -> i32:
        return self.value
