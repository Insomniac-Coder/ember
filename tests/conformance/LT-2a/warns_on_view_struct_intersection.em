#$ test: compile-pass
#$ rules: LT-2a, MAN-3
#$ warning[L3014]: view region is the intersection of 2 fields

@view
struct Pair:
    left: Span[i32]
    right: Span[i32]
