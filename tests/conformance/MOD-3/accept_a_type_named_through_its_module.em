#$ test: run-pass
#$ rules: MOD-3
#$ stdout:
#$ 7
#$ 1 2
#$ Shape.Line(5)
# `[MOD-3]` — `import support.geometry` binds `geometry`, and `geometry.T`
# names its type `T` in type position as in an expression, with arguments:
# `geometry.Pair[int]`.

import support.geometry

fn sum(p: geometry.Point) -> int:
    return p.x + p.y

pair: geometry.Pair[int] = geometry.Pair(1, 2)
shape: geometry.Shape = geometry.Shape.Line(5)
println(sum(geometry.Point(3, 4)))
println(pair.first, pair.second)
println(shape)
