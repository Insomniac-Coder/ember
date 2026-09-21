#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, OWN-8, TYP-16, ENM-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ stdout: 7

from generic_enum_derived_copy_imported_support.model import Flag

fn main():
    original = Flag[i32].On(7)
    copied = original
    match original:
        Flag.On(value): println(value)
    match copied:
        Flag.On(value): println(value)
