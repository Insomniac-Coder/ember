#$ test: parse-fail
#$ rules: GRM-8d
## "A range type is over a concrete representation", so an alias carrying
## generic parameters may not carry a range.

type Bad[T] = T in 0 ..= 1           #$ error[E2213]: a range type may not be generic
