#$ test: run-pass
#$ rules: CTL-11, GRM-44
#$ stdout: 4
#$ 5
#$ nothing
#$ 3
## `[CTL-11]` - an `if` or `while` header that opens an `Option` (`if cam.target exists as p:`) runs its
## block with the name; `else:` runs when the `Option` holds nothing.

struct Cam:
    target: Option[int]

fn main():
    cam = Cam(Some(4))
    gone = Cam(None)
    if cam.target exists as p:
        println(p)
    else:
        println("none")
    if gone.target exists as p:
        println(p)
    elif cam.target exists as q:
        println(q + 1)
    if gone.target exists as r:
        println(r)
    else:
        println("nothing")
    cur = Some(3)
    while cur exists as c:
        println(c)
        cur = None
