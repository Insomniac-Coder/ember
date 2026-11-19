#$ test: run-pass
#$ rules: CLS-4
#$ stdout: 2 1 d b
# `[CLS-4]`: `init`, `drop` and `static fn`s are each class's own: no `override`, no `E2111`.

open class Base:
    x: i32

    fn init():
        this.x = 1

    fn drop():
        print("b")

    static fn make() -> i32:
        return 1

    virtual fn go() -> i32:
        return 1

class Derived(Base):
    fn init():
        super.init()

    fn drop():
        print("d ")

    static fn make() -> i32:
        return 2

    override fn go() -> i32:
        return 2

fn main():
    d = Derived()
    print(Derived.make(), Base.make(), "")
