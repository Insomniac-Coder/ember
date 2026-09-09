#$ test: parse-pass
#$ rules: LEX-15b
## `gen` is a keyword only immediately before `fn`, so a binding may be named
## `gen` and `r#yield` names the reserved word.

fn counters():
    gen = 1
    gen = gen + 1
    r#yield = gen
    print(r#yield)
