#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, HEAP-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: em_vt_dyn_43generic_enum_mut_interface_imported_support6traits4Bump_for_43generic_enum_mut_interface_imported_support5model6SignalG2c2a47066b4fc729
#$ assert-c: em_vt_dyn_43generic_enum_mut_interface_imported_support6traits4Bump_for_43generic_enum_mut_interface_imported_support5model6SignalG2c2a47066b4fc729_slot0

from generic_enum_mut_interface_imported_support.model import Signal
from generic_enum_mut_interface_imported_support.traits import Bump

fn main():
    boxed: Box[dyn Bump] = Box(Signal[i32].Ready(7))
    println(boxed.bump())
