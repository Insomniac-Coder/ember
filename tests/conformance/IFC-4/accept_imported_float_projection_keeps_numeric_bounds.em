#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18, MOD-4, STD-27, STD-9
#$ stdout: 4.0 5.0
# A Float projection carried by an imported generic struct still supplies
# numeric methods, literals and arithmetic among other caller parameters.

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

fn calculate[P: Copy, S: DecimalSource, Q: Copy](prefix: P, source: S, suffix: Q) -> S.Item:
    item = decimal_value(source)
    return item.value.sqrt() + 1.0

fn main():
    println(calculate(true, Small(9.0f32), "small"), calculate("large", Large(16.0f64), false))
