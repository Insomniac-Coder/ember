#$ test: run-pass
#$ rules: HR-5, HR-6, HR-8, HR-9a
#$ profiles: debug, development
#$ stdout: 12
#$ 21
#$ 7
#$ assert-c: contains("int64_t em_7add_one(int64_t _1) {")
#$ assert-c: contains("return ((int64_t (*)(int64_t))em_rl_targets[")
#$ assert-c: contains("int64_t em_7add_one__rl(int64_t x) {")
#$ assert-c: contains("em_7add_one(11LL)")
#$ assert-c: !contains("static inline EMBER_MAYBE_UNUSED int64_t em_7add_one")
#$ assert-c: contains("((const struct ember_native_fn_descriptor*)em_rl_descriptors[")
#$ assert-c: contains("->ti->vtable))->slot0)(s)")
#$ assert-c: contains("EMBER_RL_MANIFEST const ember_rl_manifest em_reload_manifest = {")
#$ assert-c: contains("EMBER_RELOAD_MAGIC, EMBER_RELOAD_PROTOCOL,")
#$ assert-c: contains("{ "em_7add_one", (void (*)(void))em_7add_one__rl, (void (*)(void))em_7add_one,")
# In a reloadable build (this directory's manifest says `reload = "all"`) a call to `add_one` goes
# to its symbol, which is its thunk: the thunk loads the current target from the image's copy of
# the call table and calls it. The body has a name of its own and is never inlined, however small
# and however few its callers; a virtual method with one implementation is still called through
# its method table, since a reload may add an override. A value of the function (`held`: `add_one`
# itself is a type of its own, holding nothing, ODR-278) is the descriptor the runtime keeps one of
# for the whole process. The manifest lists the function by its mangled
# name, beside the protocol's version, in the image's own section.

fn add_one(x: int) -> int:
    return x + 1

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

open class Shape:
    virtual fn area(self) -> int:
        return 7

fn measure(s: Shape) -> int:
    return s.area()

fn main():
    println(add_one(11))
    held: fn(int) -> int = add_one
    println(apply(held, 20))
    println(measure(Shape()))
