#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-22, IFC-1, ENM-1, HEAP-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 20
#$ stdout: 22
#$ assert-c: contains(em_vt_dyn_generic_enum_multi_interface_imported_support_traits_Read_generic_enum_multi_interface_imported_support_model_Signal_i32)
#$ assert-c: contains(em_vt_dyn_generic_enum_multi_interface_imported_support_traits_Write_generic_enum_multi_interface_imported_support_model_Signal_i32)

from generic_enum_multi_interface_imported_support.model import Signal
from generic_enum_multi_interface_imported_support.traits import Read
from generic_enum_multi_interface_imported_support.traits import Write

fn main():
    reader: Box[dyn Read] = Box(Signal[i32].Ready(7))
    writer: Box[dyn Write] = Box(Signal[i32].Ready(7))
    println(reader.read())
    println(writer.write())
