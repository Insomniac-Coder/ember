#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-17, TYP-20, IFC-1, ENM-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 42

from generic_enum_interface_imported_support.model import Signal
from generic_enum_interface_imported_support.traits import Render

fn call_render[T: Render](value: T) -> i32:
    return value.render()

fn main():
    signal = Signal[i32].Value(0)
    println(call_render(signal))
