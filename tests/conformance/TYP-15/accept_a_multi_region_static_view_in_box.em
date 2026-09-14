#$ test: run-pass
#$ rules: TYP-15, LT-3, LT-14, LT-30, HEAP-1, DRP-6, TST-17
#$ stdout: 1

# A storable multi-region view has to satisfy the normal storage rule for
# every carried slot. Both `str` fields here have static provenance, so boxing
# the Pair is legal without a special multi-region storage exception.

@view
struct Pair:
    left: str
    right: str

fn main():
    pair = Pair("left", "right")
    _boxed: Box[Pair] = Box(pair)
    println(1)
