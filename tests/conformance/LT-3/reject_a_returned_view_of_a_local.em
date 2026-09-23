#$ test: compile-fail
#$ rules: LT-3, DIA-14
#$ profiles: debug
#$ error[E3060]: `tmp` does not live long enough
# D-190 — a returned view of a local is one mistake and one error: the drop
# of `tmp` at the return is not also reported as a write while borrowed.

fn view(s: Span[int]) -> Span[int]:
    return s

fn leak() -> Span[int]:
    tmp = [41]
    return view(tmp)

fn main():
    println(leak()[0])
