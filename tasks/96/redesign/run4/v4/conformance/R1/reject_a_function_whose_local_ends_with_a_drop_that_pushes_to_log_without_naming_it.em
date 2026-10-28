#$ test: compile-fail
#$ rules: R1, R5
#$ help: add it to `spawn_one`'s first line: `fn spawn_one() changes LOG:`
## R1, R5: a local's value ends with its function, and ending an object counts as a call of its `drop`,
## with what `drop`'s first line says it changes outside its object (verdict 57): `e` ends when
## `spawn_one` returns, and an `Enemy`'s `drop` pushes to `LOG`, so `spawn_one` names `LOG` (E2105).

static mut LOG: Array[String] = []

class Enemy:
    hp: int = 1

    fn drop() changes LOG:
        LOG.push(String("gone"))

fn spawn_one():                #$ error[E2105]: `spawn_one` ends an `Enemy` (`e`, made on line 17, ends when `spawn_one` returns), and an `Enemy`'s `drop` pushes to `LOG`
    e = Enemy()
    println(e.hp)

fn main() changes LOG:
    LOG.push(String("start"))
    spawn_one()
    println(LOG.len())
