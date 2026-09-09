#$ test: parse-fail
#$ rules: GRM-22
## `return`'s precedence is the lowest there is, so `yield` cannot be an
## operand (ADR-015: `E0100`, because `E0107` says "jump" and `yield` is not).

gen fn f() -> Coroutine[void]:
    x = 1 + yield 2    #$ error[E0100]: `yield` may not be an operand
