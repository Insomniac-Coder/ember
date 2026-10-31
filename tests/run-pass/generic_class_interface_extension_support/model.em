from generic_class_interface_extension_support.traits import Measure

[T]
pub class Holder:
    value: i32
    marker: T

extend[T] Holder[T] implements Measure:
    fn measure() -> i32:
        return this.value
