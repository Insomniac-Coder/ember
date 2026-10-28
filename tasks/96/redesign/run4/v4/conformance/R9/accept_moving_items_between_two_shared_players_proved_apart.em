#$ test: compile-pass
#$ rules: R9, R8, R4, R5
## R9, R8 (b): "unless the compiler knows they differ (... inside `if a is not b:`)": with the test
## the loop over `a.inventory` may add to `b.inventory` (AN-48's `transfer`). `b` is a `ref mut`
## parameter that holds a link, so the change through it is named, `changes Player` (verdict 63), and
## `main`, which calls `transfer`, names it too.

class Player:
    inventory: Array[String] = []

class Holder:
    target: shares mut Player

fn transfer(a: shares Player, ref mut b: shares mut Player) changes Player:
    if a is not b:
        for it in a.inventory:
            b.inventory.push(copy of it)   # accepted: the compiler knows they differ

fn main() changes Player:
    p = Player()
    x = Holder(target=shares mut p)
    y = Holder(target=shares mut p)
    transfer(x.target, ref mut y.target)
