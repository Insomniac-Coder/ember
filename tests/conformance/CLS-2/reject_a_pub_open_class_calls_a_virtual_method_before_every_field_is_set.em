#$ test: compile-fail
#$ rules: CLS-2
# `[CLS-2]` — any package may override a virtual method of a `pub open` class, so its `init` calls one
# only after every field is set.

pub open class Script:
    id: int

    fn init():
        this.on_make()    #$ error[E2100]
        this.id = 1

    virtual fn on_make():
        pass

fn main():
    pass
