#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-4, OWN-2
#$ profiles: debug, release, shipping
#$ assert-c: contains(em_Base_init)
#$ assert-c: contains(em_Derived_init)
#$ stdout: 11

fn base_default() -> i32:
    return 3

fn derived_default() -> i32:
    return 5

open class Base:
    value: i32 = base_default()

    fn init(mut self):
        self.value = self.value + 1

class Derived(Base):
    extra: i32 = derived_default()

    fn init(mut self):
        super.init()
        self.extra = self.extra + 2

fn main():
    item = Derived()
    println(item.value + item.extra)
