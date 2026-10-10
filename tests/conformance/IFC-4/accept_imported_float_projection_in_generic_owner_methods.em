#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18, MOD-4, STD-27
#$ stdout: true true
# The same imported Float projection must retain its numeric contract after
# a generic owner's parameter is substituted and the method slots move.

from support.formatting_projection_values import DecimalSource, decimal_value

struct Small:
    value: f32

extend Small implements DecimalSource:
    type Item = f32
    fn get(self) -> f32:
        return self.value

struct Large:
    value: f64

extend Large implements DecimalSource:
    type Item = f64
    fn get(self) -> f64:
        return self.value

struct Calculator[P: Copy]:
    marker: P

    fn increases[S: DecimalSource, Q: Copy](self, source: S, suffix: Q) -> bool:
        item = decimal_value(source)
        return item.value + 1.0 > item.value

fn main():
    calculator = Calculator[str]("owner")
    println(calculator.increases(Small(9.0f32), false), calculator.increases(Large(16.0f64), true))
