# Clause CLO-16.1: "A list, field or parameter whose functions may run on any thread says so in its
# type"; "A function of such a type holds `thread_shared` values or copies"; and the message's fix
#$ test: run-pass
#$ rules: CLO-16, THR-18, THR-20
#$ stdout: 3
#$ stdout: 1
# A copy of a number, and a function that changes a guarded module variable under its lock.

from std.sync import Lock

class Bus:
    handlers: Array[thread_shared fn()] = []

static TICK_LOCK: Lock = Lock()
static mut FRAME: int guarded by TICK_LOCK = 0

fn on_frame():
    with TICK_LOCK:
        FRAME += 1

fn main():
    bus = Bus()
    k = 3
    bus.handlers.push(owns fn() copy of k: println(k))
    bus.handlers.push(owns on_frame)
    for ref mut h in bus.handlers:
        h()
    with TICK_LOCK:
        println(FRAME)
