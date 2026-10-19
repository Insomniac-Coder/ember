#$ test: run-pass
#$ rules: OWN-7, RC-2
#$ profiles: debug, development
#$ stdout: 7
#$ done
# `[OWN-7]` (D-680) — copying a `Copy` value that holds class handles retains each one; an enum payload
# whose own retain opens another `switch` goes through a retain function of its own, so the C nests
# one level per function. Inline, 140 `Option`s deep was MSVC's "blocks nested too deeply" (C1061).
# The copy keeps its object alive when the original is overwritten.

class C:
    x: i32

fn keep(w: Option[Option[C]]) -> Option[Option[C]]:
    return w

fn deep(w: Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[C]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]) -> Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[C]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]:
    return w

fn main():
    a: Option[Option[C]] = Some(Some(C(7)))
    b = keep(a)
    a = None
    match b:
        Some(Some(c)):
            println(c.x)
        _:
            println("none")
    w: Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[Option[C]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]] = Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(C(5)))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))
    _x = deep(w)
    println("done")
