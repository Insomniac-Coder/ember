#$ test: run-pass
#$ rules: R4, R3
#$ stdout: renamed 0
## R4, verdict 53: the nearest allowed program, the lines E3031's message gives: `mine` is a string of
## `main`'s own, no part of `LOG`, so `rename` may clear `LOG` and then change it.

static mut LOG: Array[String] = []

fn rename(ref mut s: String) changes LOG:
    LOG.clear()
    s = String("renamed")

fn main() changes LOG:
    LOG.push(String("hello"))
    mine = copy of LOG[0]
    rename(ref mut mine)
    println(mine, LOG.len())
