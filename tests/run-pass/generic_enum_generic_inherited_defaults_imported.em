#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, HEAP-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: em_vt_dyn_generic_enum_generic_inherited_defaults_imported_support_traits_Child_i32_generic_enum_generic_inherited_defaults_imported_support_model_Signal_i32
#$ assert-c: em_vt_dyn_generic_enum_generic_inherited_defaults_imported_support_traits_Child_i32_generic_enum_generic_inherited_defaults_imported_support_model_Signal_i32_slot0
#$ assert-c: em_vt_dyn_generic_enum_generic_inherited_defaults_imported_support_traits_Child_i32_generic_enum_generic_inherited_defaults_imported_support_model_Signal_i32_slot1

from generic_enum_generic_inherited_defaults_imported_support.model import Signal
from generic_enum_generic_inherited_defaults_imported_support.traits import Child

fn main():
    boxed: Box[dyn Child[i32]] = Box(Signal[i32].Ready(7))
    println(boxed.parent(7) + boxed.child(7))
