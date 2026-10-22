#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, TYP-16, TYP-17, TYP-18, ENM-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 44
#$ assert-c: contains(em_36generic_enum_method_imported_support5model7Message6chooseG99f65fd6187c945d)

from generic_enum_method_imported_support.model import Message

fn main():
    message = Message[i32].Empty
    println(message.choose[i64](44i64))
