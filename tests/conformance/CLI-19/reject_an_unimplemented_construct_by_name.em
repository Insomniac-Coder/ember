#$ test: compile-fail
#$ rules: CLI-19
#$ error[E0900]: a local function with a foreign ABI is not supported yet
# A construct the specification defines and this compiler does not implement
# is rejected by name, never accepted with another meaning. (The generic local
# function and the local generator this used are built, `[GRM-28]`.)

fn main():
    extern "C" fn twice(x: i32) -> i32:
        return x * 2
    println(twice(4))
