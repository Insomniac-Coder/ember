#$ test: compile-fail
#$ rules: TXT-2
#$ error[E5063]: `StrView` would take text C never validated
#$ error[E1010]: `GREETING` was not imported
#$ warning[W5001]: macro `GREETING` is not imported
## `[TXT-2]` - nothing becomes a `StrView` without validation: a binding whose entry gives a `StrView`
## for text C made would hand foreign bytes to it unchecked, and is `E5063`.

import c "foreign_text.h" with (overlay = "overlays/text.em")

fn main():
    println(c.GREETING(1))
