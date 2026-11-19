#$ test: compile-fail
#$ rules: CLS-4
# `[CLS-4]`: `virtual` in a class that cannot be subclassed (neither `open` nor `abstract`) is `E2112`.

class Door:
    open_count: int

    fn init():
        this.open_count = 0

    virtual fn on_update():         #$ error[E2112]: `Door` cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it write `open class Door:`, or remove `virtual`
        this.open_count += 1

fn main():
    d = Door()
    d.on_update()
