# Clause BRW-11.3: "A function value given to a call that changes the call's object is refused
# likewise"
#$ test: compile-fail
#$ rules: BRW-11, CLO-17, EFF-26
# The spec's message, with this file's line numbers: `retain` changes `LOG` and `keep` adds to it.

static mut LOG: Array[String] = []

fn keep(s: String) -> bool:
    if s == "":
        LOG.push(owns String("dropped an empty line"))
        return false
    return true

fn main():
    LOG.push(owns String(""))
    LOG.push(owns String("a"))
    LOG.retain(keep)    #$ error[E3022]: `LOG.retain(keep)` changes `LOG` while it walks it, and `keep` adds to `LOG` too (its line 11); drop the lines first and push the notes after: `before = LOG.len()`, `LOG.retain(owns fn(s) => s != "")`, `dropped = before - LOG.len()`, then `for k in 0..dropped: LOG.push(owns String("dropped an empty line"))`
