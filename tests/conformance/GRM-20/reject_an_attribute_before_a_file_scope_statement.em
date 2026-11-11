#$ test: compile-fail
#$ rules: GRM-20, GRM-2, ATT-3
# `[GRM-20]` — the attributes before a statement are limited to `@parallel`, `@unroll`, `@simd` and
# `@allow`, at file scope too, where a statement follows the attribute and not a declaration.

@inline                             #$ error[E0108]: `@inline` is not permitted on a statement
count = 3

@simd                               #$ error[E0108]: `@simd` must precede a compound statement
total = 4

println(count, total)
