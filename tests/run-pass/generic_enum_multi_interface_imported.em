#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, HEAP-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 20
#$ stdout: 22
#$ assert-c: contains(em_vt_dyn_45generic_enum_multi_interface_imported_support6traits4Read_for_45generic_enum_multi_interface_imported_support5model6SignalG2c2a47066b4fc729)
#$ assert-c: contains(em_vt_dyn_45generic_enum_multi_interface_imported_support6traits5Write_for_45generic_enum_multi_interface_imported_support5model6SignalG2c2a47066b4fc729)

from generic_enum_multi_interface_imported_support.model import Signal
from generic_enum_multi_interface_imported_support.traits import Read
from generic_enum_multi_interface_imported_support.traits import Write

fn main():
    reader: Box[dyn Read] = Box(Signal[i32].Ready(7))
    writer: Box[dyn Write] = Box(Signal[i32].Ready(7))
    println(reader.read())
    println(writer.write())
