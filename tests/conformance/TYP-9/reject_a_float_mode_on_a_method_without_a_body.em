#$ test: compile-fail
#$ rules: TYP-9, ATT-1, ATT-6, DIA-14
#$ profiles: debug
#$ error[E0104]: `@fastmath` applies to a method with a body
#$ error[E0104]: `@fp` applies to a method with a body
#$ error[E0104]: `@fp` needs `contract`: `@fp(contract)`
# A float mode governs the body it is written on (`[TYP-9]`): on an interface
# method with none it would be ignored, which `[ATT-6]` never allows; each
# implementation carries its own. A wrong `@fp` on an interface's default is
# one mistake and one error, however many types take the default
# (`[DIA-14]`).

interface Shader:
    @fastmath
    fn shade(self, x: f64) -> f64

interface Lit:
    @fp(contract)
    fn light(self) -> f64

interface Shape:
    @fp(fast)
    fn area(self) -> f64:
        return 0.0

class Circle implements Shape:
    r: f64

class Square implements Shape:
    s: f64

fn main():
    pass
