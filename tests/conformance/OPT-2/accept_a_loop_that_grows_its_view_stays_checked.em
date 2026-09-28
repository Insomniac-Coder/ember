#$ test: run-pass
#$ rules: OPT-2, RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 0 1 2 0 1 2
#$ stdout: 6
#$ assert-c-count: contains("ember_vec_push_i64(") == 3
#$ assert-c-count: contains("ember_ck_add_i64(") == 1
# Pushing to the view the loop indexes changes its length, so no entry test
# could cover the loop: it is left as it was, checked, and not copied. The
# second loop pushes through another handle of the same class, to a list that
# could be (and here is) the one it indexes: also left checked. Neither loop
# is copied: three pushes in all. Of the two `+`, only `total + value` is
# checked: `i + 1` cannot overflow for an `i` below 3 (`[RNG-4]`).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn main():
    xs: Array[int] = [0, 1, 2]
    for i in 0..3:
        xs.push(xs[i])
    println(xs[0], xs[1], xs[2], xs[3], xs[4], xs[5])

    bag = Bag()
    other = bag
    for i in 0..3:
        bag.items.push(i + 1)
    total = 0
    for i in 0..3:
        value = bag.items[i]
        other.items.push(value)
        total = total + value
    println(total)
