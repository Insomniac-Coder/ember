#$ test: compile-fail
#$ rules: R12, R4
#$ help: LOG.retain(fn(s) => s != "")
## R12 (rules.md lines 672-675), R4 (rules.md lines 268-277): a function value given to a call counts as
## an argument that changes every place its type names, so `LOG.retain(keep)` with a `keep` that changes
## `LOG` names one place twice (E3022). rules.md section 3's line 1008 message, word for word.

static mut LOG: Array[String] = []

fn keep(s: String) changes LOG -> bool:
    if s != "":
        return true
    LOG.push(String("dropped an empty line"))
    return false

fn main() changes LOG:
    LOG.push(String("a"))
    LOG.push(String(""))
    LOG.retain(keep)    #$ error[E3022]: `LOG.retain(keep)` changes `LOG` while it walks it, and `keep` changes `LOG` too (it pushes a note for each line it drops); drop the lines first and push the notes after: `before = LOG.len()`, `LOG.retain(fn(s) => s != "")`, `dropped = before - LOG.len()`, then `for k in 0..dropped: LOG.push(String("dropped an empty line"))`
    println(LOG.len())
