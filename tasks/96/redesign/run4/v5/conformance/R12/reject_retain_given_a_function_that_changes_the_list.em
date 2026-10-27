#$ test: compile-fail
#$ rules: BRW-11
#$ help: LOG.retain(fn(s) => s != "")
## design rules: R12, R4
## R12 (rules.md lines 720-722), R4 (lines 286-296): "For R4's overlap rule a function value given to a
## call counts as an argument that changes every place its functions change ..., so `LOG.retain(keep)`,
## where `keep` pushes a note to `LOG`, is refused (E3022)"; what `keep` changes is worked out from its
## body (verdict 86). rules.md section 3's line 1060 `retain` message, in this program's lines. The
## nearest allowed program is accept_retain_then_push_the_notes.

static mut LOG: Array[String] = []

fn keep(s: String) -> bool:
    if s != "":
        return true
    LOG.push(String("dropped an empty line"))
    return false

fn main():
    LOG.push(String("a"))
    LOG.push(String(""))
    LOG.retain(keep)    #$ error[E3022]: `LOG.retain(keep)` changes `LOG` while it walks it, and `keep` adds to `LOG` too (line 15); drop the lines first and push the notes after: `before = LOG.len()`, `LOG.retain(fn(s) => s != "")`, `dropped = before - LOG.len()`, then `for k in 0..dropped: LOG.push(String("dropped an empty line"))`
    println(LOG.len())
