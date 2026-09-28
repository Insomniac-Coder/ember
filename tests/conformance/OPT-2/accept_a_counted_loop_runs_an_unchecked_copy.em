#$ test: run-pass
#$ rules: OPT-2, OPT-3, RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 4950
#$ stdout: 9801
#$ stdout: 5050
#$ stdout: 4950
#$ assert-c-count: contains("ember_panic_bounds(") == 5
#$ assert-c-count: contains("ember_ck_add_i64(") == 12
#$ assert-c-count: contains("ember_vec_push_i64(") == 4
# Each loop that indexes indexes a view at `i + c` whose base and length it
# cannot change. The first loop's own test keeps `i` below `len(xs)`, which
# the range facts see (`[RNG-4]`): its check goes and the loop is not copied.
# Each other one gets an entry test and an unchecked copy: its body appears
# twice and its bounds checks once (2 + 1 + 1 + 1 = 5). Checked `+`: one in
# the first loop, two in each copy of the second and of the third (there the
# `- 1` after a checked `+ 1`, and the `+ 0`, cannot overflow), one in each
# copy of the last, and one outside (1 + 4 + 4 + 2 + 1 = 12). Pushing to `ys`
# while reading `bag.items` changes only `ys`, so that loop is copied too
# (3 + 1 = 4 pushes); the loops that only push have no index and are left
# alone.

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn main():
    xs: Array[int] = []
    for i in 0..100:
        xs.push(i)

    total = 0
    for i in 0..len(xs):
        total = total + xs[i]
    println(total)

    pairs = 0
    for i in 1..100:
        pairs = pairs + xs[i - 1] + xs[i]
    println(pairs)

    upto = 0
    for i in 0..=99:
        upto = upto + xs[i] + 1 - 1 + 0
    println(upto + 100)

    bag = Bag()
    for i in 0..100:
        bag.items.push(i)
    ys: Array[int] = []
    for i in 0..100:
        ys.push(bag.items[i])
    held = 0
    for i in 0..100:
        held = held + bag.items[i]
    println(held)
