#$ test: compile-fail
#$ rules: FFI-11b
# `[FFI-11b]` — `count: inout_count(items)` is C's pointer to the count,
# written `mut count` of an integer type, and `items` is
# `nullable count(count)` and `exclusive`: the function takes
# `Option[MutSpan[T]]`. Its call returns the count, after C's own result when
# there is one (`accept_inout_count_with_a_result_returns_both`).

unsafe extern "C":
    @ffi(param(count, inout_count(items)), param(items, borrowed, count(count), nullable, exclusive))
    safe fn by_value(count: u32, items: Option[MutSpan[i32]])    #$ error[E5002]: `inout_count(items)` needs `mut count` of an integer type

    @ffi(param(count, inout_count(items)), param(items, borrowed, count(count), exclusive))
    safe fn not_nullable(mut count: u32, items: MutSpan[i32])    #$ error[E5002]: `inout_count(items)` needs `param(items, borrowed, count(count), nullable, exclusive)`

fn main():
    pass
