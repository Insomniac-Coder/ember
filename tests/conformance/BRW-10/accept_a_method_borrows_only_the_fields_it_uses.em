#$ test: run-pass
#$ rules: BRW-10
#$ stdout: a bb ccc 3
# `[BRW-10]` — a call to a struct's method changes only the fields its body
# changes, through the methods of the same kind it calls: `w.bump()` may run
# while a loop iterates `w.names`. (A view the method returns points into the
# whole object, `[LT-1]`: LT-1/reject_changing_another_field_while_a_methods_view_lives.)

struct Words:
    names: Array[String]
    count: int = 0

    fn bump():
        this.add(1)

    fn add(n: int):
        this.count += n

fn main():
    w = Words(["a", "bb", "ccc"])
    for name in w.names:
        w.bump()
        print(name, "")
    println(w.count)
