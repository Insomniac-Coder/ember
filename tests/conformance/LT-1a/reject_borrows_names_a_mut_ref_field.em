#$ test: compile-fail
#$ rules: LT-1a, BRW-6
# `[LT-1a]` (ODR-196) — a result read through a `ref mut` field borrows `self` as well: the
# reborrow continues through a unique reference (`[BRW-6]`), so it cannot name the field alone,
# and one iterator cannot hand out two mutable items. `@borrows(self.owner)` here is `E3062`, at
# the `return` whose value borrows `self`.

@view
struct ValuesMut:
    owner: ref mut Array[int]
    at: int

    fn next() -> Option[ref mut int] from owner:
        if this.at >= this.owner.len():
            return None
        i = this.at
        this.at += 1
        return Some(ref mut this.owner[i])    #$ error[E3062]: `next` returns a view of `this`, but its first line says `from owner`

fn main():
    ys = [1, 2]
    v = ValuesMut(ref mut ys, 0)
    match v.next():
        Some(r):
            println(r)
        None:
            pass
