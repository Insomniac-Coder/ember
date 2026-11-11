#$ test: compile-fail
#$ rules: BRW-7, OWN-3, FN-1
#$ error[E3040]: `xs` was handed over to `moved` on line 12 and is used on line 13; to keep both, write `moved = copy of xs` (a copy of your own) or `moved = shares xs` (two owners of one value)
#$ error[E3040]: `ys` was handed over to `take` on line 23 and is used on line 24; to keep both, hand over a copy of your own: `copy of ys`

## `[BRW-7]` — no borrow of a moved place. Taking an address does not consume,
## so a borrow is not a move; but a reference into memory whose owner gave it
## away dangles, which is a separate rule from `[OWN-3]` and a separate code.

fn borrow_after_move() -> int:
    xs: Array[i32] = Array[i32]()
    moved: Array[i32] = xs
    r: ref Array[i32] = ref xs
    return moved.len() + r.len()

## `[FN-1]` — `owned` consumes. Until the mode was threaded through argument
## lowering, every argument was borrowed and this compiled.
fn take(owns zs: Array[i32]) -> int:
    return zs.len()

fn use_after_owned_call() -> int:
    ys: Array[i32] = Array[i32]()
    n: int = take(owns ys)
    return n + ys.len()

fn main():
    println(borrow_after_move())
    println(use_after_owned_call())
