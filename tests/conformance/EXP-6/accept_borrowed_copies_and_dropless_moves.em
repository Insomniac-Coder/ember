#$ test: run-pass
#$ rules: EXP-6, BRW-1, FN-1
# What `E3013` is not: a blanket ban on touching borrowed data. `[BRW-1]`
# lets the owner read and copy while shared borrows are live, and a move of a
# borrowed value that owns nothing has no second destruction to happen — so
# all three stay legal beside the rejects. Deliberately absent here: moving a
# droppable field out of an *owned* struct, which `[EXP-6]` allows but whose
# scope-end drop is a separate open defect (D-042), not this one.

struct Panel:
    pub w: i32
    pub h: i32

fn retitle(p: Panel, t: i32) -> Panel:
    q = p
    q.w = t
    return q

struct Outer:
    pub n: i32

fn get_n(o: Outer) -> i32:
    return o.n

fn main():
    o = Outer(1)
    r: ref Outer = ref o
    println(r.n)
    println(get_n(o))
    println(o.n)
    p = retitle(Panel(1, 2), 9)
    println(p.w)
    println(p.h)
#$ stdout: 1
#$ 1
#$ 1
#$ 9
#$ 2
