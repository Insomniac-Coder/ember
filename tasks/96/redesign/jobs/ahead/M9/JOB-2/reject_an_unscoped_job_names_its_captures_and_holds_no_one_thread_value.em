# Clause JOB-2.1: "`jobs.submit` outside a scope takes a function any thread may run, `pub fn
# submit(owns f: thread_shared once fn()) -> JobHandle:`, as `thread.spawn` does (`[THR-10]`,
# `[THR-17]`): a function that holds a one-thread value is `E7015`, and a name used with no capture
# word `E7004`."
#$ test: compile-fail
#$ rules: JOB-2, THR-17, THR-2
# No text in the spec for these programs: code only.

import std.jobs

class Player:
    hp: int = 5

fn main():
    p = Player()
    h = jobs.submit(owns fn(): println(p.hp))    #$ error[E7004]
    jobs.wait(h)
    s = shares Player()
    k = jobs.submit(owns fn() owns s: println(s.hp))    #$ error[E7015]
    jobs.wait(k)
