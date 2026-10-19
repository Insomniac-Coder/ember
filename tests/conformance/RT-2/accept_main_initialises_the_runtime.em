#$ test: run-pass
#$ rules: RT-2
#$ profiles: debug, development, release
#$ stdout: ready
#$ assert-c: contains(ember_rt_init(&cfg);)
#$ assert-c: !contains(__attribute__((constructor)))
#$ assert-c: !contains(.CRT$XCU)
# `[RT-2]` — the runtime has no global constructors: the generated `main` starts it by calling
# `ember_rt_init`, and the program's own C asks for no code to run before `main`. (The
# runtime's files are checked for the same in `ember_driver`'s `runtime_has_no_global_constructors`.)

fn main():
    println("ready")
