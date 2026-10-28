#$ test: compile-fail
#$ rules: R9, R8, R4
## R9, R8 (c): two `shares` links may lead to one player, so a loop over one link's list may not add
## to the list reached through the other until the program proves the two differ (AN-48's `transfer`
## without its test, still refused).

class Player:
    inventory: Array[String] = []

class Holder:
    target: shares mut Player

fn transfer(a: shares Player, ref mut b: shares mut Player):
    for it in a.inventory:
        b.inventory.push(copy of it)       #$ error[E3020]: `a` and `b` may be the same player, and this loop walks `a.inventory` while adding to `b.inventory`; test first: `if a is not b:`

fn main():
    p = Player()
    x = Holder(target=shares mut p)
    y = Holder(target=shares mut p)
    transfer(x.target, ref mut y.target)
