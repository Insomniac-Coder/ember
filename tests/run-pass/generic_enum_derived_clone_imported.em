#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, OWN-8, TYP-16, ENM-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 9
#$ assert-c: contains(em_43generic_enum_derived_clone_imported_support5model5Entry5cloneG2c2a47066b4fc729)

from generic_enum_derived_clone_imported_support.model import Entry

fn main():
    original = Entry[i32].Value(9)
    cloned = original.clone()
    match cloned:
        Entry.Value(value): println(value)
