#$ test: compile-fail
#$ rules: R2, R5
#$ help: add it to `swap_in`'s first line: `fn swap_in() changes LOG:`
## R2, R5: `=` on a name that has a value ends its old value, and ending an object counts as a call of
## its `drop` (verdict 57): the old `Res` ends on line 18, and a `Res`'s `drop` pushes to `LOG`, so
## `swap_in` names `LOG` (E2105).

static mut LOG: Array[String] = []

class Res:
    name: String

    fn drop() changes LOG:
        LOG.push(String("ended"))

fn swap_in():                  #$ error[E2105]: `swap_in` ends a `Res` (`a = Res(...)` on line 18 replaces the old one, and `a` ends when `swap_in` returns), and a `Res`'s `drop` pushes to `LOG`
    a = Res(name=String("one"))
    a = Res(name=String("two"))
    println(a.name)

fn main() changes LOG:
    LOG.push(String("start"))
    swap_in()
    println(LOG.len())
