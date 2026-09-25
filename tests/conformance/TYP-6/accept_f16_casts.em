#$ test: run-pass
#$ rules: TYP-6, TYP-5
#$ profiles: debug, release, shipping
#$ stdout: 0.099975586 0.0999755859375 true
#$ inf 2048.0 2052.0 -2.75 1.0
#$ 255 65504 -32768 0 2147483647
#$ 1000.0 inf 1
# D-316 — `x as T` to and from `f16` (`[TYP-6]`): exact from it, rounded
# once to it, ties to even; to an integer it rounds toward zero and
# saturates, NaN to 0. It was a C cast of the stored bits.

fn main():
    x: f16 = 0.1
    wide: f32 = x
    println(wide, x as f64, (x as f64) == 0.0999755859375)
    n = 70000
    halfway: f32 = 1.0 + 1.0 / 2048.0
    println(n as f16, 2049 as f16, 2051 as f16, (-2.75) as f16, halfway as f16)
    big = f16.MAX
    println(big as u8, big as int, (-big) as i16, f16.NAN as int, f16.INF as i32)
    w: i128 = 1000
    u: u64 = 18446744073709551615
    y: f16 = 1.5
    println(w as f16, u as f16, y as i128)
