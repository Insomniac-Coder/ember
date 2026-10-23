#$ test: compile-fail
#$ rules: GRM-28, PHIL-12
## `[GRM-28]` — what is not built yet is refused, never ignored: a local function with a foreign
## ABI (`fn_decl`'s `extern "C"`), named.

fn main():
    extern "C" fn twice(x: i32) -> i32:    #$ error[E0900]: a local function with a foreign ABI is not supported yet
        return x * 2
    println(twice(4))
