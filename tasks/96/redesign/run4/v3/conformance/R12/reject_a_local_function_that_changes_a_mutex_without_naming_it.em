#$ test: compile-fail
#$ rules: R12, R5, R15
#$ help: fn add_one() with thread_shared stats changes Mutex[Stats]:
## R12 (rules.md lines 509-512), verdict 29: a change of the value inside a `Mutex` is named by the
## `Mutex`'s type (`changes Mutex[Stats]`); verdict 51: a local function takes the same capture list and
## `changes` as a lambda. Message in the shape of rules.md section 3's E2105 lock message.

import std.thread
from std.sync import Mutex

struct Stats:
    total: int = 0

fn main() changes Mutex[Stats]:
    stats = thread_shared Mutex(Stats())
    fn add_one() with thread_shared stats:    #$ error[E2105]: `add_one` changes the `Stats` in `stats` through `g` (under `stats`'s lock); say so on its first line: `fn add_one() with thread_shared stats changes Mutex[Stats]:`
        with g = stats.lock():
            g.total += 1
    t = thread.spawn(add_one)
    t.join()
    with g = stats.lock():
        println(g.total)
