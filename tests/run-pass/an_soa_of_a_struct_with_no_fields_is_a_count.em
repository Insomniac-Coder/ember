#$ test: run-pass
#$ stdout: 3 2 0
# `[SOA-1]` — an `SoA` of a struct with no fields has no columns: it counts its elements. Its C
# helpers named parameters they did not use and allocated a block for no column, which gcc's
# `-Wextra` refuses under the C gate (`[TST-27]`).
struct Marker:
    pass

fn main():
    ms = SoA[Marker].with_capacity(1)
    ms.push(Marker())
    ms.push(Marker())
    ms.push(Marker())
    a = ms.len()
    _gone = ms.swap_remove(0)
    b = ms.len()
    ms.clear()
    println(a, b, ms.len())
