#$ test: compile-fail
#$ rules: CLS-4
# `[CLS-4]`: an `override` has the first line of the method it replaces: the same parameter names, words and types,
# the same words on `this` and the same result (`E2113`).

open class Event:
    code: i32

    fn init(code: i32):
        this.code = code

class KeyEvent(Event):
    key: i32

    fn init(code: i32, key: i32):
        this.key = key
        super.init(code)

open class Handler:
    virtual fn on(e: Event):
        println(e.code)

    virtual fn rename(old: i32, fresh: i32) -> i32:
        return old + fresh

    virtual fn fill(ref mut out: Array[i32]):
        out[0] += 1

    virtual fn size() -> i32:
        return 1

    virtual fn finish() owns this:
        pass

    virtual fn pair(a: i32, b: i32) -> i32:
        return a + b

class KeyLog(Handler):
    override fn on(e: KeyEvent):    #$ error[E2113]: `KeyLog.on` takes `e: KeyEvent`, and the `Handler.on` it overrides takes `e: Event`; write `override fn on(e: Event):` and test the kind inside: `if e is KeyEvent as k:`
        println(e.key)

    override fn rename(old: i32, next: i32) -> i32:     #$ error[E2113]: `KeyLog.rename` names its parameter `next`, and the `Handler.rename` it overrides names it `fresh`; write `override fn rename(old: i32, fresh: i32) -> i32:`
        return old + next

    override fn fill(out: Array[i32]):  #$ error[E2113]: `KeyLog.fill` takes `out: Array[i32]`, and the `Handler.fill` it overrides takes `ref mut out: Array[i32]`; write `override fn fill(ref mut out: Array[i32]):`
        println(out.len())

    override fn size() -> bool:     #$ error[E2113]: `KeyLog.size` returns `bool`, and the `Handler.size` it overrides returns `i32`; write `override fn size() -> i32:`
        return true

    override fn finish():           #$ error[E2113]: `KeyLog.finish` takes no word on `this`, and the `Handler.finish` it overrides takes `owns this`; write `override fn finish() owns this:`
        pass

    override fn pair(a: i32) -> i32:    #$ error[E2113]: `KeyLog.pair` takes 1 parameter, and the `Handler.pair` it overrides takes 2; write `override fn pair(a: i32, b: i32) -> i32:`
        return a

fn main():
    pass
