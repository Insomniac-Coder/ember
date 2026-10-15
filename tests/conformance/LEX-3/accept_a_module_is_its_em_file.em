#$ test: run-pass
#$ rules: LEX-3, MOD-1
#$ stdout: from the .em file
# `[LEX-3]` — the source extension is `.em`: `support.greeting` is `support/greeting.em`, and the
# `support/greeting.txt` beside it, which declares the same function, is never read.

import support.greeting

fn main():
    println(greeting.greeting())
