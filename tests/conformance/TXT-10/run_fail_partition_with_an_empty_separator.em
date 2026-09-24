#$ test: run-fail
#$ rules: TXT-10, ERR-13
#$ panics: partition: empty separator
# `[TXT-10]` — `partition` is Python's, whose empty separator is an error; a
# caller's bug panics (`[ERR-13]`).

println("ab".partition(""))
