#$ test: run-pass
#$ rules: OPT-1, WK-1
#$ profiles: debug, release
#$ stdout: 6
#$ 4
#$ 5
#$ assert-c: contains("(struct em_obj_6Passed*)ember_obj_new_frame(&_fo")
#$ assert-c: contains("((struct em_obj_8Returned*)ember_obj_new(&em_ti_8Returned))")
#$ assert-c: !contains("(struct em_obj_8Returned*)ember_obj_new_frame(")
#$ assert-c: contains("((struct em_obj_8Weakened*)ember_obj_new(&em_ti_8Weakened))")
#$ assert-c: !contains("(struct em_obj_8Weakened*)ember_obj_new_frame(")
# `[OPT-1]`'s escape analysis follows a handle through calls and stops at what it cannot prove.
# A `Passed` given to a function that gives it to another, neither keeping it, is in the frame.
# A `Returned` whose method returns `self` may outlive the function, and a `Weakened` with a `Weak`
# made of it would leave the weak handle pointing into a frame that is gone: both are on the heap.

class Passed:
    v: int

class Returned:
    v: int

    fn same(self) -> Returned:
        return self

class Weakened:
    v: int

fn deeper(p: Passed) -> int:
    return p.v

fn through(p: Passed) -> int:
    return deeper(p)

fn passed() -> int:
    p = Passed(6)
    return through(p)

fn returned() -> int:
    r = Returned(4)
    s = r.same()
    return s.v

fn weakened() -> int:
    w = Weakened(5)
    _weak = Weak(w)
    return w.v

fn main():
    println(passed())
    println(returned())
    println(weakened())
