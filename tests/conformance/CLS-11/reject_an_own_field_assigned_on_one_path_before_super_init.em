#$ test: compile-fail
#$ rules: CLS-11

open class Base:
    fn init(self):
        pass

class Derived(Base):
    tag: String

    fn init(self, owned tag: String):
        if tag.len() > 3:
            self.tag = tag
        super.init() #$ error[E2102]: field `tag` is not assigned on every path before `super.init(...)`

fn main():
    _d = Derived("t")
