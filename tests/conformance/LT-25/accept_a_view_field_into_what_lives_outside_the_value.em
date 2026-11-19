# Clause LT-25: "A view field may not point into another field of the value that contains it (shape
# B5)" -- a view field may point into what lives outside the value, and an owning field beside it
# holds something else.
#$ test: run-pass
#$ rules: LT-25, TYP-15, LT-18
#$ stdout: 7 3
#$ stdout: 8

struct Mixed:
    first: Span[int]
    values: Array[int]

fn main():
    outside = [7, 8]
    item = Mixed(first=outside.as_span(), values=[1, 2, 3])
    println(item.first[0], item.values.len())
    item.first = outside[1..]
    println(item.first[0])
