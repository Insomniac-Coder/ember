#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, HEAP-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: em_vt_dyn_56generic_enum_generic_inherited_defaults_imported_support6traits5ChildG2c2a47066b4fc729_for_56generic_enum_generic_inherited_defaults_imported_support5model6SignalG2c2a47066b4fc729
#$ assert-c: em_vt_dyn_56generic_enum_generic_inherited_defaults_imported_support6traits5ChildG2c2a47066b4fc729_for_56generic_enum_generic_inherited_defaults_imported_support5model6SignalG2c2a47066b4fc729_slot0
#$ assert-c: em_vt_dyn_56generic_enum_generic_inherited_defaults_imported_support6traits5ChildG2c2a47066b4fc729_for_56generic_enum_generic_inherited_defaults_imported_support5model6SignalG2c2a47066b4fc729_slot1

from generic_enum_generic_inherited_defaults_imported_support.model import Signal
from generic_enum_generic_inherited_defaults_imported_support.traits import Child

fn main():
    boxed: Box[dyn Child[i32]] = Box(Signal[i32].Ready(7))
    println(boxed.parent(7) + boxed.child(7))
