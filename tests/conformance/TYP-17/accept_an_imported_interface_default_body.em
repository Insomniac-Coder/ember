#$ test: run-pass
#$ rules: TYP-17, IFC-1, MOD-4

from support.defaults import Named

struct Item implements Named:
    marker: i32

fn main():
    item = Item(1)
    println(item.value())
#$ stdout: 41
