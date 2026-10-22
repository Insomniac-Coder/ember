#$ test: run-pass
#$ rules: CG-C-6, SIMD-5
#$ profiles: release, debug
#$ stdout: [3, 6, 9, 12]
#$ assert-c: contains("    EMBER_VECTORISE")
#$ assert-c-count: contains("    EMBER_VECTORISE") == 1
# `[CG-C-6]`: a loop in vectorisable form (`[SIMD-5]`, here proved by `@simd(assert)`, which would
# be `E4020` otherwise) is preceded by the host compiler's vectorisation pragma, `EMBER_VECTORISE`
# (`ember_rt.h`: clang's `loop vectorize(enable)`, GCC's `ivdep`, MSVC's `loop(ivdep)`), written
# directly before its `for`, in every profile. Its checked multiply is grouped (`[SIMD-7]`), and it
# is that grouped copy, with no check left in it, that the pragma leads. The program's other loops
# (pushing, printing) are not in vectorisable form and get no pragma.

fn scale(xs: Span[int], mut out: MutSpan[int], k: int):
    @simd(assert)
    for i in 0..xs.len():
        out[i] = xs[i] * k

fn main():
    a = [1, 2, 3, 4]
    b = [0, 0, 0, 0]
    scale(a, b, 3)
    println(b)
