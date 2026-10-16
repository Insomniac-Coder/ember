#$ test: parse-fail
#$ rules: GRM-22, GRM-16
## `yield` is a jump expression at `return`'s precedence, the lowest there is,
## so it cannot be an operand: `E0107`, as `a + return b` is (`[GRM-16]`;
## ADR-193 replaced ADR-015's `E0100`, from when `yield` had a resume type).

gen fn f() -> Generator[int]:
    x = 1 + yield 2    #$ error[E0107]: `yield` may not be an operand
