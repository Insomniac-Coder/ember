#$ test: run-pass
#$ rules: DRP-1, DIA-7, PHIL-8a
#$ stdout:
#$ 1
#$ 2

import std.mem

struct R:
    pub value: i32

    fn drop(mut self):
        println(self.value)

fn main():
    r = R(1)
    mem.drop(r)
    println(2)
