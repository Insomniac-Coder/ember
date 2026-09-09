#$ test: parse-pass
#$ rules: GRM-8d, RNG-1
## A `range_clause` is admitted where `type_alias` appears as an `item`, and
## the production is LL(2): `for x in xs` is untouched by it.

type Roughness = f32 in 0.0 ..= 1.0
type Percent   = u8  in 0 .. 101

fn walk(xs: Array[i32]):
    for x in xs:
        print(x)
