#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 1 2
## Verdict 41, R12 (rules.md lines 724-728): `fn tick() changes world:` is the line
## `fn tick() changes FRAME, LOG:`, checked the same way both ways; `note` names only what it changes.

static mut FRAME: int = 0
static mut LOG: Array[String] = []

world: changelist = (FRAME, LOG)

fn tick() changes world:
    FRAME += 1
    LOG.push(String("tick"))

fn note() changes LOG:
    LOG.push(String("note"))

fn main() changes world:
    tick()
    note()
    println(FRAME, LOG.len())
