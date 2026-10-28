#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 3
## R15 and AN-78: "A plain capture (the real variable) is allowed only for `callback(borrowed)`, which C
## calls only during the call"; the function is a named local function (verdict 51) that names the
## variable it changes (verdict 22), `main`'s own, which needs no clause (R5).

import c "./kept_events.c" as raw

unsafe extern "C":
    @ffi(param(cb, callback(borrowed, user_data=ud)), link_name="ke_each")
    safe fn each(cb: extern "C" fn(*mut void), ud: *mut void, times: i32)

fn main():
    count = 0
    fn bump() changes count:
        count += 1
    each(bump, 3)
    println(count)
