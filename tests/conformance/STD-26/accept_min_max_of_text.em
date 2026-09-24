#$ test: run-pass
#$ rules: STD-26, TYP-37
#$ stdout: a b m 1
# `[STD-26]` — `min`, `max` and `clamp` take any `Ord` type, and text is one
# (`[TYP-37]`): a `str` compares by its bytes.

println(min("b", "a"), max("b", "a"), clamp("q", "c", "m"), min(2, 1))
