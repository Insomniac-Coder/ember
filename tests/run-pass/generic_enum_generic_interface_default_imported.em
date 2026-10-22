#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, HEAP-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: em_vt_dyn_55generic_enum_generic_interface_default_imported_support6traits6AnswerG2c2a47066b4fc729_for_55generic_enum_generic_interface_default_imported_support5model6SignalG2c2a47066b4fc729
#$ assert-c: em_vt_dyn_55generic_enum_generic_interface_default_imported_support6traits6AnswerG2c2a47066b4fc729_for_55generic_enum_generic_interface_default_imported_support5model6SignalG2c2a47066b4fc729_slot0

from generic_enum_generic_interface_default_imported_support.model import Signal
from generic_enum_generic_interface_default_imported_support.traits import Answer

fn main():
    boxed: Box[dyn Answer[i32]] = Box(Signal[i32].Ready(7))
    println(boxed.answer(7))
