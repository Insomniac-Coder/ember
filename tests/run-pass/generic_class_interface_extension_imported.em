#$ test: run-pass
#$ rules: MOD-1, MOD-3, TYP-16, IFC-1, TYP-22
#$ profiles: debug, release, shipping
#$ stdout: 42

from generic_class_interface_extension_support.model import Holder
from generic_class_interface_extension_support.traits import Measure

fn measure(value: ref dyn Measure) -> i32:
    return value.measure()

fn main():
    holder = Holder[bool](42, false)
    println(measure(ref holder))
