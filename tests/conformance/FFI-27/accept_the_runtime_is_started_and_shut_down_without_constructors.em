#$ test: run-pass
#$ rules: FFI-27, RT-2
#$ profiles: debug, development, release
#$ stdout: ready
#$ assert-c: contains(ember_rt_init(&cfg);)
#$ assert-c: contains(ember_rt_shutdown();)
#$ assert-c: !contains(__attribute__((constructor)))
#$ assert-c: !contains(.CRT$XCU)
# `[FFI-27]` — the runtime `ember_rt` is a C11 static library with no global constructors: a host
# calls `ember_rt_init(&cfg)` once, and `ember_rt_shutdown()` at the end. The generated `main` is
# such a host. (The C host fixtures of `ember_driver`'s tests call the rest: thread attach and
# `ember_rt_object_release`.)

fn main():
    println("ready")
