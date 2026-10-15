#$ test: run-pass
#$ rules: DET-1
#$ stdout: 2.5
# `[DET-1]` — exact arithmetic and `sqrt`, correctly rounded everywhere, are deterministic.

@deterministic
fn mean(a: f64, b: f64) -> f64:
    return (a + b) / 2.0 + (4.0).sqrt() - 2.0

fn main():
    println(mean(2.0, 3.0))
