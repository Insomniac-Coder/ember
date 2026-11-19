#$ test: compile-fail
#$ rules: BRW-1, BRW-2, LT-1
#$ error[E3063]: which ends when the function returns

## Milestone M2 (Part XX §3). The specification writes it over `Array[i32]`
## and `xs[0]`, which needs `Index` to return a `ref` — that arrives with
## block D, when the containers are written in Ember. The rule under test is
## the same one: a borrow of a parameter may be returned because the caller
## owns what it points at, and a borrow of a local may not, because its
## storage ends with the frame.

## Fine: the region is tied to the parameter by `[LT-1]`'s elision.
fn first(ref xs: String) -> ref String:
    return xs

fn bad() -> ref i32:
    n: i32 = 1
    return ref n

fn main():
    n: String = "7".to_string()
    r: ref String = ref n
    println(first(r))
