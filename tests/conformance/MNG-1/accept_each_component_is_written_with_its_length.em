#$ test: run-pass
#$ rules: MNG-1, CG-C-10
#$ stdout: 9 70
#$ assert-c: contains("int64_t em_3lib3m_x1f(int64_t n) {")
#$ assert-c: contains("int64_t em_3lib1m3x_f(int64_t n) {")
#$ assert-c: contains("{"em_3lib3m_x1f", "lib.m_x.f", ")
#$ assert-c: contains("{"em_3lib1m3x_f", "lib.m.x_f", ")
# `[MNG-1]` — a symbol is `em_` and each component of the item's path written as its length in
# decimal and then its text: module `lib/m_x.em`'s `f` is `em_3lib3m_x1f` and module `lib/m.em`'s
# `x_f` is `em_3lib1m3x_f`, the specification's own pair. With `_` between the components both were
# `em_lib_m_x_f`, one C function defined twice, and a program holding both was an internal compiler
# error (D-809). Each is called from two places, so neither is inlined into `main`, and each frame
# is named by its own path (`[CG-C-10]`).

import lib.m_x
import lib.m

fn main():
    println(m_x.f(3) + m_x.f(4), m.x_f(3) + m.x_f(4))
