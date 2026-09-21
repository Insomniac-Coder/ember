#$ test: run-pass
#$ rules: MOD-1, MOD-2, MOD-3, OWN-8, TYP-16, ENM-1, MONO-1
#$ profiles: debug, release, shipping
#$ stdout: 7
#$ assert-c: contains(em_generic_enum_derived_clone_noncopy_imported_support_model_Token_clone)
#$ assert-c: contains(Entry_generic_enum_derived_clone_noncopy_imported_support_model_Token_clone)

from generic_enum_derived_clone_noncopy_imported_support.model import Entry
from generic_enum_derived_clone_noncopy_imported_support.model import Token

fn main():
    original = Entry[Token].Value(Token(7))
    cloned = original.clone()
    match cloned:
        Entry.Value(token): println(token.value)
