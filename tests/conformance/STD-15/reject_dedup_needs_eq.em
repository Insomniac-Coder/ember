#$ test: compile-fail
#$ rules: STD-15, GRM-34
#$ note: `dedup` needs `Token` to implement `Eq`
# `[STD-15]` — `dedup` compares neighbours, so it exists only for elements
# that implement `Eq`; the error says which bound is missing.

@no_derive(Eq)
struct Token:
    id: int

ts = [Token(1)]
ts.dedup()   #$ error[E1010]: `Array[Token]` has no method named `dedup`
