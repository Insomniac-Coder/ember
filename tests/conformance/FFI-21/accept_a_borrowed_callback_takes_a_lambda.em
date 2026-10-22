#$ test: run-pass
#$ rules: FFI-21, FFI-11, FFI-10
#$ profiles: debug, development, release
#$ stdout: 10 4
#$ stdout: 24
#$ stdout: 6 seen 3
#$ stdout: 20
# `[FFI-21]`, `[FFI-11]` — a C function taking a function pointer and the user data C hands back
# to it: `callback(borrowed, user_data=data)` makes the pointer the callable a caller passes, a
# lambda capturing what any call argument may (here the caller's locals, written through), and
# the user data leaves the callable signature: the wrapper gives C the callable's address, and the
# trampoline the contract generates calls the lambda directly. The declaration writes the pointer
# as C has it, the user data among its parameters (first here, last and an integer below), and
# no allocation is made for a borrowed callback. A local function (a named closure) and a
# function are callables too. No `unsafe`.

from std.ffi import c_int

import c "./visits.c" as raw

unsafe extern "C":
    @ffi(param(values, borrowed, count(count)), param(visit, callback(borrowed, user_data=data)), link_name="visit_each")
    safe fn visit_each(values: Span[c_int], count: usize, visit: extern "C" fn(*mut void, c_int), data: *mut void)
    @ffi(param(values, borrowed, count(count)), param(map, callback(borrowed, user_data=data)), link_name="sum_mapped")
    safe fn sum_mapped(values: Span[c_int], count: usize, map: extern "C" fn(c_int, isize) -> c_int, data: isize) -> c_int

fn triple(value: c_int) -> c_int:
    return value * 3

fn main():
    values: Array[c_int] = [1, 2, 3, 4]
    total = 0
    seen = 0
    fn add(value: c_int):
        total += value
        seen += 1
    visit_each(values.as_span(), add)
    println(total, seen)
    bias: c_int = 2
    println(sum_mapped(values.as_span(), fn(value: c_int) -> c_int => value * bias + 1))
    largest = 0
    count = 0
    fn keep_largest(value: c_int):
        if value > largest:
            largest = value
        count += 1
    visit_each(values[0..3], keep_largest)
    println(largest * 2, "seen", count)
    println(sum_mapped(values[0..2], triple) + sum_mapped(values[2..4], triple) - 10)
