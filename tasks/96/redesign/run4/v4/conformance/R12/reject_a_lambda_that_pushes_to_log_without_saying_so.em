#$ test: compile-fail
#$ rules: R12, R5
#$ help: fn() changes LOG: LOG.push(String("clicked"))
## R12 (rules.md lines 608, 622-625), verdict 17: "A plain `fn()` changes nothing at all"; a lambda that
## changes a module variable says so on its first line, checked against its body (E2105). The message is
## rules.md section 3's E2105 row (line 989), word for word.

static mut LOG: Array[String] = []

fn main() changes LOG:
    click = fn(): LOG.push(String("clicked"))    #$ error[E2105]: this function pushes to `LOG`; say so on its first line: `fn() changes LOG: LOG.push(String("clicked"))`
    click()
    println(LOG.len())
