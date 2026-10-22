#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, OWN-8, TYP-16, ENM-1, MONO-1
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains(em_51generic_enum_derived_clone_noncopy_imported_support5model5Token5clone)
#$ assert-c: contains(em_51generic_enum_derived_clone_noncopy_imported_support5model5Entry5cloneG63d4d374f4d940dc)

from generic_enum_derived_clone_noncopy_imported_support.model import Entry
from generic_enum_derived_clone_noncopy_imported_support.model import Token

fn main():
    original = Entry[Token].Value(Token(7))
    cloned = original.clone()
    match cloned:
        Entry.Value(token): println(token.value)
