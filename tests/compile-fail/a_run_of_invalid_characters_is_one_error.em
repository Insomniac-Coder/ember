#$ test: compile-fail
#$ rules: DIA-20, DIA-14
# `[DIA-20]` — a run of characters with no meaning in Ember is one
# diagnostic, not one per character; `[DIA-14]` — the parser adds nothing
# for the token the lexer rejected.

fn main():
    x = 1 ``` 2             #$ error[E0100]: unexpected characters "```"
    y = 3 $ 4               #$ error[E0100]: unexpected character `$`
