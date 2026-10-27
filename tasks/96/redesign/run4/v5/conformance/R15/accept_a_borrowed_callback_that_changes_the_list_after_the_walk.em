#$ test: run-pass
#$ rules: R15, R12, R8
#$ stdout: a
#$ stdout: 2
## R12: the line E3020 gives: the C call that runs `grow` comes after the walk (completeness G23's
## nearest allowed).

import c "./kept_events.c" as raw

unsafe extern "C":
    @ffi(param(cb, callback(borrowed, user_data=ud)), link_name="ke_each")
    safe fn each(cb: extern "C" fn(*mut void), ud: *mut void, times: i32)

fn main():
    names: Array[String] = [String("a")]
    fn grow():
        names.push(String("b"))
    for n in names:
        println(n)
    each(grow, 1)
    println(names.len())
