#$ test: run-pass
#$ rules: IFC-4
#$ profiles: debug, development, release
#$ stdout: 7 0
# D-381 — a default method calls an associated function through its
# interface's associated type by its bare name (`Item.default()`), as it names
# the type elsewhere; the bound `type Item: Default` provides it. `Nums`'s
# copy calls `int`'s.

from std.core import Default

interface Source:
    type Item: needs Default
    fn get(i: int) -> Item
    fn size() -> int

    fn first_or_default() -> Item:
        if this.size() == 0:
            return Item.default()
        return this.get(0)

struct Nums:
    xs: Array[int]

extend Nums implements Source:
    type Item = int

    fn get(i: int) -> int:
        return this.xs[i]

    fn size() -> int:
        return this.xs.len()

fn main():
    println(Nums([7, 8]).first_or_default(), Nums([]).first_or_default())
