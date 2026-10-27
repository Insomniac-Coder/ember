#$ test: compile-fail
#$ rules: R4, R12
#$ help: LOG.retain(fn(s) => s != "")
## R4, R12: a function value given to a call counts as an argument that changes every place the
## functions it may run change, worked out from their bodies (verdict 86), so `keep`, which pushes to
## `LOG`, may not be given to `LOG.retain` (rules section 3, E3022).

static mut LOG: Array[String] = []

fn keep(s: String) -> bool:
    if s == "":
        LOG.push(String("dropped an empty line"))
    return s != ""

fn main():
    LOG.push(String("a"))
    LOG.push(String(""))
    LOG.retain(keep)    #$ error[E3022]: `LOG.retain(keep)` changes `LOG` while it walks it, and `keep` changes `LOG` too (it pushes a note for each line it drops); drop the lines first and push the notes after: `before = LOG.len()`, `LOG.retain(fn(s) => s != "")`, `dropped = before - LOG.len()`, then `for k in 0..dropped: LOG.push(String("dropped an empty line"))`
    println(LOG.len())
