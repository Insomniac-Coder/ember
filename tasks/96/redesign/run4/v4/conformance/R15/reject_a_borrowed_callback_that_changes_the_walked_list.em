#$ test: compile-fail
#$ rules: R15, R12, R8
## R12: "a `callback(borrowed)` parameter runs it during the call, as `for_each` does", so the call
## makes the function's changes now; inside a walk of what it changes it is E3020 (R8 (c); completeness
## G23; the message is written to R18 in the form of section 3's `for_each` one).

import c "./kept_events.c" as raw

unsafe extern "C":
    @ffi(param(cb, callback(borrowed, user_data=ud)), link_name="ke_each")
    safe fn each(cb: extern "C" fn(*mut void), ud: *mut void, times: i32)

fn main():
    names: Array[String] = [String("a")]
    fn grow() changes names:
        names.push(String("b"))
    for n in names:
        println(n)
        each(grow, 1)    #$ error[E3020]: this loop walks `names` (line 17), and `each(grow, 1)` (line 19) runs `grow` during the call (`callback(borrowed)`), which pushes to `names`; call it after the loop: move line 19 below the loop
    println(names.len())
