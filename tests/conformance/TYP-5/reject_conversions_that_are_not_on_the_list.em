#$ test: compile-fail
#$ rules: TYP-5
## `[TYP-5]` - only the listed conversions happen at a coercion site: not narrowing, not signed to unsigned, not
## a float to an integer or the other way, not to or from `bool` or `char`, not an `Option[T]` to a `T`.

fn takes_i32(x: i32): println(x)
fn takes_f32(x: f32): println(x)
fn takes_char(c: char): println(c)

fn main():
    run(5, 7)

fn run(wide: i64, s: i16):
    a: i32 = wide                    #$ error[E2020]: expected `i32`, found `i64`
    u: u16 = s                       #$ error[E2020]: expected `u16`, found `i16`
    d: f64 = 1.5
    f: f32 = d                       #$ error[E2020]: expected `f32`, found `f64`
    n: i32 = 3
    g: f64 = n                       #$ error[E2020]: expected `f64`, found `i32`
    i: int = 1
    b: bool = i                      #$ error[E2020]: expected `bool`, found `i64`
    code: u32 = 65
    ch: char = code                  #$ error[E2020]: expected `char`, found `u32`
    o: Option[int] = Some(1)
    k: int = o                       #$ error[E2020]: expected `i64`, found `Option[i64]`
    takes_i32(wide)                  #$ error[E2020]: expected `i32`, found `i64`
    takes_f32(d)                     #$ error[E2020]: expected `f32`, found `f64`
    takes_char(code)                 #$ error[E2020]: expected `char`, found `u32`
    println(a, u, f, g, b, ch, k)
