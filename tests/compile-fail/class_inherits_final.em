#$ test: compile-fail
#$ rules: CLS-4
#$ error[E2020]: class `Child` cannot inherit from final class `Base`

class Base:
    value: i32

class Child(Base):
    extra: i32

fn main():
    println(1)
