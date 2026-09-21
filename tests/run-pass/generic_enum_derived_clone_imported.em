#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, OWN-8, TYP-16, ENM-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 9
#$ assert-c: contains(em_generic_enum_derived_clone_imported_support_model_Entry_i32_clone)

from generic_enum_derived_clone_imported_support.model import Entry

fn main():
    original = Entry[i32].Value(9)
    cloned = original.clone()
    match cloned:
        Entry.Value(value): println(value)
