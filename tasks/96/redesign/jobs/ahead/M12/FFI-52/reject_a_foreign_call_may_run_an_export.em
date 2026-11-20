# Clause FFI-52.1: "every `@export` and `@export_table` function whose thread contract (`[FFI-33]`)
# allows it"; its message for an export
#$ test: compile-fail
#$ rules: FFI-52, FFI-33, BCK-3
# The export `log_line` adds to `LOG`; `host_poll`, a C function with no mark, may run it. The
# message's tail names an overlay this test does not have (`host.overlay.em`): the claim stops
# before it. "exported, line N" is read as the attribute's line.

static mut LOG: Array[String] = []

# SAFETY: nothing else in the process defines `log_line`
@unsafe(export("log_line", threads=main))
pub extern "C" fn log_line():
    LOG.push(owns String("from C"))

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    safe fn host_poll()

fn main():
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        host_poll()    #$ error[E3020]: `host_poll()` (line 23) may run `log_line` (exported, line 12), which adds to `LOG` (line 14) while this loop walks `LOG` (line 21); call `host_poll()` before or after the loop
