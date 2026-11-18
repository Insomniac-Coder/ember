#$ test: run-pass
#$ rules: EXP-4
#$ stdout: drop 4
#$ stdout: arm 4
#$ stdout: in ann
#$ stdout: drop list
#$ stdout: after
#$ stdout: arm ann
#$ stdout: drop list
#$ stdout: end
# `[EXP-4]` — a temporary created while evaluating the subject of a `match` is dropped before the
# chosen arm runs, unless the arm's pattern names what it holds (`names().first()` keeps the list
# while `p` views a string in it, to the end of that block or arm only).

struct T:
    id: int

    fn drop():
        println(f"drop {this.id}")

    fn val() -> int:
        return this.id

struct L:
    items: Array[String]

    fn drop():
        println("drop list")

    fn first() -> Option[ref String]:
        return this.items.get(0)

fn mk(i: int) -> T:
    return T(id=i)

fn names() -> L:
    return L(items=["ann", "bob"])

fn main():
    match mk(4).val():
        4:
            println("arm 4")
        _:
            pass
    if names().first() exists as p:
        println("in", p)
    println("after")
    match names().first():
        Some(q):
            println("arm", q)
        None:
            pass
    println("end")
