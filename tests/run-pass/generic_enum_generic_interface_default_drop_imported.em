#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, DRP-6, HEAP-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 8
#$ stdout: 7
#$ assert-c: contains(em_vt_dyn_60generic_enum_generic_interface_default_drop_imported_support6traits6AnswerG9da2466bd67cbe4c)
#$ assert-c: contains(em_vt_dyn_60generic_enum_generic_interface_default_drop_imported_support6traits6AnswerG9da2466bd67cbe4c_for_60generic_enum_generic_interface_default_drop_imported_support5model6SignalG9da2466bd67cbe4c_slot0)
#$ assert-c: contains(ember_box_new_copy)

from generic_enum_generic_interface_default_drop_imported_support.model import Signal
from generic_enum_generic_interface_default_drop_imported_support.model import Token
from generic_enum_generic_interface_default_drop_imported_support.traits import Answer

fn main():
    boxed: Box[dyn Answer[Token]] = Box(Signal[Token].Ready(Token(7)))
    println(boxed.answer(Token(8)))
