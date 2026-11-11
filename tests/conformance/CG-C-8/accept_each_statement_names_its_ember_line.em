#$ test: run-pass
#$ rules: CG-C-8
#$ profiles: debug, development
#$ stdout: 6
#$ assert-c: contains("#line 12 \"accept_each_statement_names_its_ember_line.em\"")
#$ assert-c: contains("#line 13 \"accept_each_statement_names_its_ember_line.em\"")
#$ assert-c: contains("#line 14 \"accept_each_statement_names_its_ember_line.em\"")
# `[CG-C-8]` — every emitted statement is preceded by a `#line` naming the Ember construct that
# produced it: the three statements of `main` below are lines 12, 13 and 14.

fn main():
    total = 1
    for i in range(1, 3):
        total += i + 1
    println(total)
