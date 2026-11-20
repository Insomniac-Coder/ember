# Clause BRW-21.3: "It works out from each function's body which of its view parameters, `this`
# included, it uses after a line that may end or move what they view (a reshape; the end of an
# object, `[RC-5]`), through its own lines and the same fact of the functions it calls (a call
# through a function value, or through an interface, `dyn` or virtual method, by every function the
# program can put there, `[EFF-1]`)"
#$ test: compile-fail
#$ rules: BRW-21, BCK-8, EFF-1, EFF-24, DSP-5, RC-5
# Four ways for the end to come from elsewhere: a callee (`wipe`), a stored function value (`HOOK`
# holds `wipe`), an override (`Clearing.flush`: the call is given a `Sink`, but the program has the
# override), and the end of an object (`strike` clears `ENEMIES`, which ends the `Enemy` that `a`
# reads). The spec gives no text for these: code only.

static mut LOG: Array[String] = []

fn wipe():
    LOG.clear()

static mut HOOK: fn() = wipe

fn note(s: StrView):
    wipe()
    println(s)

fn note_hook(s: StrView):
    HOOK()
    println(s)

open class Sink:
    fn init():
        pass

    virtual fn flush():
        pass

class Clearing(Sink):
    fn init():
        super.init()

    override fn flush():
        LOG.clear()

fn note_to(s: StrView, k: Sink):
    k.flush()
    println(s)

class Enemy:
    hp: int = 3

static mut ENEMIES: Array[Enemy] = []

fn strike(a: Enemy):
    ENEMIES.clear()
    println(a.hp)

fn main():
    LOG.push(owns String("a"))
    ENEMIES.push(owns Enemy())
    note(LOG[0])    #$ error[E3031]
    note_hook(LOG[0])    #$ error[E3031]
    note_to(LOG[0], Sink())    #$ error[E3031]
    strike(ENEMIES[0])    #$ error[E3031]
