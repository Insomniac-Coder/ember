#$ test: compile-fail
#$ rules: LEX-3
#$ error[E1010]: cannot find module `support.notes`
# `[LEX-3]` — `support/notes.txt` holds Ember source, but no `.em` file has the module's name,
# so there is no module `support.notes`.

import support.notes

fn main():
    println(notes.note())
