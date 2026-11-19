#$ rules: DIA-12
#$ test: compile-fail
#$ error[E1010]: `valeu`
#$ help: the nearest name is `value`
from support.base import Base

class Derived(Base):
    current: i32 = 0

fn read(item: Derived) -> i32:
    return item.valeu

fn main():
    println(0)
