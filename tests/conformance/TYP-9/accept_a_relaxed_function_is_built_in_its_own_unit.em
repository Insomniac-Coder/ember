#$ test: run-pass
#$ rules: TYP-9, TYP-9b, CG-C-11, ATT-6
#$ profiles: debug, release, shipping
#$ assert-c: contains("/* module: accept_a_relaxed_function_is_built_in_its_own_unit, its `contract` functions ([CG-C-11]) */")
#$ assert-c: contains("/* module: accept_a_relaxed_function_is_built_in_its_own_unit, its `fastmath` functions ([CG-C-11]) */")
#$ assert-c: contains("#pragma STDC FP_CONTRACT ON")
#$ assert-c: contains("#pragma fp_contract(on)")
#$ assert-c-count: contains("double em_fused(double _1, double _2, double _3) {") == 1
#$ assert-c-count: contains("double em_total(ember_vec* _1, ember_vec* _2) {") == 1
#$ assert-c-count: contains("const uint8_t em_interface_id_Area = UINT8_C(0);") == 1
#$ assert-c-count: contains("extern const uint8_t em_interface_id_Area;") == 2
#$ assert-c-order: "its `contract` functions" then "double em_fused(double _1, double _2, double _3) {"
#$ assert-c-order: "its `fastmath` functions" then "double em_total(ember_vec* _1, ember_vec* _2) {"
#$ assert-c-order: "its `fastmath` functions" then "1.25e0"
#$ stdout: 7.0 7.0
#$ stdout: 5.25 true 3
# `[TYP-9]` — `@fp(contract)` permits a multiply and an add to fuse, and
# `@fastmath` every relaxation, within the function only. `[CG-C-11]` builds
# each relaxed function in a C file of its own, compiled with that mode's
# flags: the main file declares `fused` and `total` and defines neither. A
# lambda takes the mode of the function it is written in (ODR-090): its
# `1.25` is in the `@fastmath` file, not before it. What a
# program has one of (an interface's id, a function value) is defined once,
# in the main file, so a function value made in the `@fastmath` file equals
# one made in the main file, and an interface call from it finds its method.
# Every value here is exact, so it is the same whichever way it is computed.

interface Area:
    fn area(self) -> f64

class Square implements Area:
    side: f64

    fn area(self) -> f64:
        return self.side * self.side

class Circle implements Area:
    r: f64

    fn area(self) -> f64:
        return self.r * self.r * 3.0

fn inc(x: int) -> int:
    return x + 1

@fp(contract)
fn fused(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

fn strict(a: f64, b: f64, c: f64) -> f64:
    return a * b + c

@fastmath
fn total(xs: Array[f64], shapes: Array[Area]) -> f64:
    scaled = fn(x: f64) -> f64:
        return x * 1.25
    sum = 0.0
    for x in xs:
        sum += x
    for shape in shapes:
        sum += scaled(shape.area())
    return sum

@fastmath
fn chosen() -> fn(int) -> int:
    return inc

fn main():
    println(fused(2.0, 3.0, 1.0), strict(2.0, 3.0, 1.0))
    shapes: Array[Area] = [Square(0.5), Circle(0.5)]
    f: fn(int) -> int = inc
    println(total([1.5, 2.5], shapes), f == chosen(), chosen()(2))
