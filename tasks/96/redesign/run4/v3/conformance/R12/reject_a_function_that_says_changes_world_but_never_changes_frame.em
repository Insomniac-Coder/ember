#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn note() changes LOG:
## Verdict 41, R12 (rules.md lines 607-611): a named change list stands for its names on a first line,
## strictly: a function says `changes world` only if it changes everything the list names (E2105).
## rules.md line 610's message, in this program's names.

static mut FRAME: int = 0
static mut LOG: Array[String] = []

world: changelist = (FRAME, LOG)

fn tick() changes world:
    FRAME += 1
    LOG.push(String("tick"))

fn note() changes world:    #$ error[E2105]: `note` says `changes world`, but it never changes `FRAME`; write `fn note() changes LOG:`
    LOG.push(String("note"))

fn main() changes world:
    tick()
    note()
    println(FRAME, LOG.len())
