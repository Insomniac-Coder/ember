#$ test: compile-fail
#$ rules: LEX-3
#$ error[E1060]: no module named 'support.notes'
# `[LEX-3]` — `support/notes.txt` holds Ember source, but no `.em` file has the module's name,
# so there is no module `support.notes`.

import support.notes

fn main():
    println(notes.note())
