# Clause CLS-12.2: "`@cached` is `E2117`, each message giving the `@cached` line to remove: on a
# method of a struct or an enum (...); on a method with parameters (...); on one whose body changes
# anything (...); and on one whose body reads a `shares`, `shares mut`, `thread_shared`, `weakref`,
# `sharedref mut` or `Cell` field (...); and on a `virtual` method or an `override`"; "For a method
# that reads a value its other owners may change: E2117"
#$ test: compile-fail
#$ rules: CLS-12, EFF-27
#$ error[E2117]: a `Vec2` has no hidden state to keep a result in; keep it in a field of a class, or remove `@cached` (line 20)
#$ error[E2117]: `@cached` keeps one result, and `radius` takes `scale`; remove `@cached` (line 30), or keep the results in a field: `radii: Map[f64, f64]`
#$ error[E2117]: a cached method keeps one result, so it only reads, and `radius` changes `this.verts` (its line 40); remove `@cached` (line 38)
#$ error[E2117]: `label` is `virtual`, and a kept result follows one body; keep it in a method no subclass replaces, or remove `@cached` (line 47)
#$ error[E2117]: `radius` reads `mesh`, which its other owners may change without `Mesh` seeing it, so a kept result could be stale; remove `@cached` (line 57)
# The spec's messages, with this file's line numbers. They are claimed for the file, not for a line:
# each message names the `@cached` line, and the spec does not say where the refusal stands (Q2).
# The cases of a field another thread may change (an `Atomic`, a lock, `guarded by`) are M9's.

struct Vec2:
    x: f64

    @cached
    fn length() -> f64:
        return this.x

class Mesh:
    size: f64 = 1.0

class Circle:
    r: f64 = 1.0

    @cached
    fn radius(scale: f64) -> f64:
        return this.r * scale

class Ring:
    r: f64 = 1.0
    verts: Array[f64] = []

    @cached
    fn radius() -> f64:
        this.verts.push(owns this.r)
        return this.r

open class Widget:
    fn init():
        pass

    @cached
    virtual fn label() -> int:
        return 1

class Model:
    mesh: shares Mesh

    fn init(owns mesh: shares Mesh):
        this.mesh = mesh

    @cached
    fn radius() -> f64:
        return this.mesh.size

fn main():
    println(Vec2(1.0).length())
    println(Circle().radius(2.0))
    println(Ring().radius())
    println(Widget().label())
    m = shares Mesh()
    println(Model(m).radius())
