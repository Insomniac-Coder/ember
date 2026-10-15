#$ test: compile-fail
#$ rules: TYP-2
#$ profiles: debug
#$ error[E2020]: cannot be cast to `bool`
#$ error[E3100]: requires `unsafe`
#$ error[E3100]: `read` needs an `unsafe` block
# `[TYP-2]` — a `bool` other than 0 or 1 can be made only in `unsafe`: no cast turns a number
# into a `bool`, and seeing bytes as `bool`s goes through a raw pointer, whose cast and read
# are both `unsafe`.

fn main():
    x: int = 2
    b = x as bool
    values: Array[u8] = Array[u8]()
    values.push(2)
    m = values.as_mut_span()
    p: *mut u8 = m.as_mut_ptr()
    q = p as *mut bool
    println(b, read(q, 0))
