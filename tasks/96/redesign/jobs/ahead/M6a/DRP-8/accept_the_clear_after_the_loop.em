# Clause DRP-8.1: "clear after the loop: move line 5, `enemies.clear()`, below the loop"; "A `drop`
# may change things outside its object (a log line, leaving a list)."
#$ test: run-pass
#$ rules: DRP-8, DIA-25
#$ stdout: start
#$ stdout: enemy gone
#$ stdout: enemy gone
# The message's fix compiles, and the two `drop`s run at the clear.

static mut LOG: Array[String] = []

fn main():
    enemies = [Enemy(), Enemy()]
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
    enemies.clear()
    for line in LOG:
        if line != "start":
            println(line)

class Enemy:
    hp: int = 1

    fn drop():
        LOG.push(owns String("enemy gone"))
