#$ test: compile-fail
#$ rules: TYP-9, TYP-9b, ATT-6
#$ profiles: debug
#$ error[E0104]: `@fp` needs `contract`: `@fp(contract)`
#$ error[E0104]: `@fp` needs `contract`: `@fp(contract)`
#$ error[E0104]: `@fastmath` takes no arguments
# `@fp` names one mode, `contract` (`[TYP-9b]`), and `@fastmath` takes none:
# anything else is an attribute with no meaning, which `[ATT-6]` never lets
# pass silently.

@fp(fast)
fn a(x: f64) -> f64:
    return x

@fp
fn b(x: f64) -> f64:
    return x

@fastmath(all)
fn c(x: f64) -> f64:
    return x

fn main():
    println(a(1.0), b(1.0), c(1.0))
