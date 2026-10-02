#$ test: run-fail
#$ rules: TXT-10, ERR-13
#$ panics: split: the separator is empty
# `[TXT-10]` — `split(sep)` is Python's, whose empty separator is an error; a
# caller's bug panics (`[ERR-13]`).

for part in "abc".split(""):
    println(part)
