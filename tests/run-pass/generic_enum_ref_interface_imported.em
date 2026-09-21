#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, DRP-6, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 7
#$ assert-c: em_vt_dyn_generic_enum_dynamic_box_imported_support_traits_Render_generic_enum_dynamic_box_imported_support_model_Signal_generic_enum_dynamic_box_imported_support_model_Token
#$ assert-c: em_vt_dyn_generic_enum_dynamic_box_imported_support_traits_Render_generic_enum_dynamic_box_imported_support_model_Signal_generic_enum_dynamic_box_imported_support_model_Token_slot0

from generic_enum_dynamic_box_imported_support.model import Signal
from generic_enum_dynamic_box_imported_support.model import Token
from generic_enum_dynamic_box_imported_support.traits import Render

fn render(value: ref dyn Render) -> i32:
    return value.render()

fn main():
    signal = Signal[Token].Ready(Token(7))
    println(render(ref signal))
