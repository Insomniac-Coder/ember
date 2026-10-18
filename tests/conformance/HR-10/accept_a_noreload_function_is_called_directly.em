#$ test: run-pass
#$ rules: HR-10, HR-10a
#$ profiles: debug, release
#$ stdout: 12
#$ assert-c: !contains("em_total__rl")
#$ assert-c: contains("int64_t em_weigh__rl(int64_t _1) {")
#$ assert-c: contains("em_weigh(")
#$ assert-c: contains("{ "em_total", NULL, NULL, NULL, 0, UINT64_C(")
#$ assert-c: contains("EMBER_RL_FN_NORELOAD }")
# In a reloadable build a `@noreload` function has no thunk: it is called directly and may be
# inlined, and the manifest lists it with its body's hash, so a reload that changes its body is
# refused. It calls a reloadable function through that function's thunk, here once per call.

@noreload
fn total(a: int, b: int) -> int:
    return weigh(a) + weigh(b)

fn weigh(x: int) -> int:
    return x * 2

fn main():
    println(total(2, 4))
