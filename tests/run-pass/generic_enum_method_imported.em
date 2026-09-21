#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-17, TYP-18, ENM-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 44
#$ assert-c: contains(em_generic_enum_method_imported_support_model_Message_i32_choose_)

from generic_enum_method_imported_support.model import Message

fn main():
    message = Message[i32].Empty
    println(message.choose[i64](44i64))
