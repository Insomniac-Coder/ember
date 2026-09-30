#$ test: run-pass
#$ rules: RNG-8, TYP-39, LEX-19, STD-9
#$ stdout: 0.25 3 0.2 <0.25> <3>   7 [0.25, 0.75] (3, 0.25)
# ODR-093 — a range value shows as its representation's value: printed, in
# an f-string (a spec applies as to the representation), through a
# `Display` bound, and inside an aggregate. Printing one was `E0900`.

type Unit = f64 in 0.0 ..= 1.0
type Level = i32 in 0 ..= 10

fn show[T: Display](x: T) -> String:
    return f"<{x}>"

fn main():
    a = Unit.clamped(0.25)
    b = Unit.clamped(0.75)
    l = Level.clamped(3)
    seven = Level.clamped(7)
    xs: Array[Unit] = [a, b]
    println(a, l, f"{a:.1f}", show(a), show(l), f"{seven:>3}", xs, (l, a))
