#$ test: run-pass
#$ rules: STA-1, STA-3, GRM-4
#$ stdout: 3 9 2
#$ stdout: start
#$ stdout: tick
#$ stdout: tick
#$ stdout: tick
#$ stdout: 3
#$ assert-c: int64_t em_5FRAME = ((int64_t)0LL);
#$ assert-c: !contains("const int64_t em_5FRAME")
# `[STA-1]` — a `static mut` is a module variable a function changes by its name, with no word
# and no `unsafe`: `FRAME += 1` and `HOME.x = …` assign the module's variable (`[GRM-4]`), and a
# list one grows. One of a plain type is a plain global the program writes, never constant data.

struct Point:
    x: int
    y: int

static mut FRAME: int = 0
static mut HOME: Point = Point(x=0, y=2)
static mut LOG: Array[String] = [String("start")]

fn tick():
    FRAME += 1
    HOME.x = HOME.x + FRAME
    LOG.push(owns String("tick"))

fn main():
    tick()
    tick()
    tick()
    HOME.x = HOME.x + 3
    println(FRAME, HOME.x, HOME.y)
    for word in LOG:
        println(word)
    println(LOG.len() - 1)
