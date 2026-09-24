#$ test: run-pass
#$ rules: IFC-2, TYP-20
#$ stdout: 8 2
# `[IFC-2]`'s way to add a method to another package's type: an interface of
# one's own, implemented for it (`[TYP-20]`).

interface Twice:
    fn twice(self) -> i64

interface Second[T]:
    fn second(self) -> T

extend i64 implements Twice:
    fn twice(self) -> i64:
        return self * 2

extend[T: Copy] Array[T] implements Second[T]:
    fn second(self) -> T:
        return self[1]

x = 4
println(x.twice(), [1, 2, 3].second())
