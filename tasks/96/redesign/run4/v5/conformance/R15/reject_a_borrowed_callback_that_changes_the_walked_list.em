#$ test: compile-fail
#$ rules: BRW-18, CTL-2
## design rules: R15, R12, R8
## R12: "a `callback(borrowed)` parameter runs it during the call, as `for_each` does", so the call
## makes the function's changes now; inside a walk of what it changes it is E3020 (R8 (c); completeness
## G23; the message is in the form of section 3's `for_each` one, naming the chain, R18). The nearest
## allowed program is `accept_a_borrowed_callback_that_changes_the_list_after_the_walk.em`.

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
        each(grow, 1)    #$ error[E3020]: `each(grow, 1)` (line 20) pushes to `names` (through `grow`, line 17, which C runs during the call: `callback(borrowed)`) while this loop walks `names` (line 18); call it after the loop: move line 20 below the loop
    println(names.len())
