#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, DRP-6, MONO-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: 7
#$ assert-c: em_vt_dyn_41generic_enum_dynamic_box_imported_support6traits6Render_for_41generic_enum_dynamic_box_imported_support5model6SignalGc5e577ec125bc548
#$ assert-c: em_vt_dyn_41generic_enum_dynamic_box_imported_support6traits6Render_for_41generic_enum_dynamic_box_imported_support5model6SignalGc5e577ec125bc548_slot0

from generic_enum_dynamic_box_imported_support.model import Signal
from generic_enum_dynamic_box_imported_support.model import Token
from generic_enum_dynamic_box_imported_support.traits import Render

fn render(value: ref dyn Render) -> i32:
    return value.render()

fn main():
    signal = Signal[Token].Ready(Token(7))
    println(render(ref signal))
