#$ test: compile-fail
#$ rules: CLS-2, CLS-11
# `[CLS-2]` — the check of a virtual call reads every override's body.

open class Base:
    id: int

    fn init():
        this.on_make()    #$ error[E2100]
        this.id = 1

    virtual fn on_make():
        pass

class Reader(Base):
    override fn on_make():
        println(this.id)

fn main():
    pass
