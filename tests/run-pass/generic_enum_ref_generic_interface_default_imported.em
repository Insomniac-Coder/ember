#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: em_vt_dyn_generic_enum_generic_interface_default_imported_support_traits_Answer_i32_generic_enum_generic_interface_default_imported_support_model_Signal_i32
#$ assert-c: em_vt_dyn_generic_enum_generic_interface_default_imported_support_traits_Answer_i32_generic_enum_generic_interface_default_imported_support_model_Signal_i32_slot0

from generic_enum_generic_interface_default_imported_support.model import Signal
from generic_enum_generic_interface_default_imported_support.traits import Answer

fn answer(value: ref dyn Answer[i32]) -> i32:
    return value.answer(7)

fn main():
    signal = Signal[i32].Ready(7)
    println(answer(ref signal))
