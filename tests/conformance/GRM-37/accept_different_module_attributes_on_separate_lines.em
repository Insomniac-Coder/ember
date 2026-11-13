#$ test: run-pass
#$ rules: GRM-37, ATT-1
#$ stdout: 1
#! language "0.9.20"
#! module overflow(saturate)
#! module deterministic
#! threads any

## `[GRM-37]` — `#! module name(args)` applies one attribute to this source module, with the name and
## argument grammar of `@name(args)`; different module attributes may appear on separate directive
## lines (`#! module overflow(saturate)`), before the imports and the items.

fn main():
    println(1)
