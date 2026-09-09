#$ test: parse-fail
#$ rules: GRM-8d
## Admitted only at item level: an associated type in an `interface` is not one.

interface I:
    type Item = f32 in 0.0 ..= 1.0   #$ error[E2213]: a range type may only be declared at item level
