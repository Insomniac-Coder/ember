#$ test: compile-fail
#$ rules: MOD-7, CLS-7, EXC-1
## `[MOD-7]` applies to class fields as well as struct fields. A class-valued
## field receiver must not turn `pub(read)` into a writable escape hatch.

from support.class_health import ClassHealth, make, heal

fn main():
    h = make(10)
    println(h.value)
    heal(h, 5)
    h.value = 99          #$ error[E1050]: `support.class_health.ClassHealth.value` is read-only outside its module
