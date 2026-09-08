#$ test: compile-fail
#$ rules: BRW-7, OWN-3, FN-1
#$ error[E3050]: `xs` is borrowed after it has been moved out of
#$ error[E3040]: `ys` has been moved out of

## `[BRW-7]` — no borrow of a moved place. Taking an address does not consume,
## so a borrow is not a move; but a reference into memory whose owner gave it
## away dangles, which is a separate rule from `[OWN-3]` and a separate code.

fn borrow_after_move() -> usize:
    xs: Array[i32] = Array[i32]()
    moved: Array[i32] = xs
    r: ref Array[i32] = ref xs
    return moved.len() + r.len()

## `[FN-1]` — `owned` consumes. Until the mode was threaded through argument
## lowering, every argument was borrowed and this compiled.
fn take(owned zs: Array[i32]) -> usize:
    return zs.len()

fn use_after_owned_call() -> usize:
    ys: Array[i32] = Array[i32]()
    n: usize = take(ys)
    return n + ys.len()

fn main():
    println(borrow_after_move())
    println(use_after_owned_call())
