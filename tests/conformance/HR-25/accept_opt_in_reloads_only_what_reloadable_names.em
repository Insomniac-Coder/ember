#$ test: run-pass
#$ rules: HR-25, MAN-7
#$ profiles: debug, development
#$ stdout: 21
#$ assert-c: contains("int64_t em_tuned__rl(int64_t x) {")
#$ assert-c: !contains("em_helper__rl")
#$ assert-c: !contains("em_main__rl")
#$ assert-c: contains("int64_t em_Gauge_read__rl(")
#$ assert-c: !contains("em_Gauge_reset__rl")
# This directory's manifest says `reload = "opt-in"`: only what `@reloadable` names is reached
# through a thunk. On a type the attribute is its methods', and a method's own attribute wins.

@reloadable
fn tuned(x: int) -> int:
    return x * 2

fn helper(x: int) -> int:
    return x + 1

@reloadable
class Gauge:
    level: int = 4

    fn read(self) -> int:
        return self.level

    @noreload
    fn reset(mut self):
        self.level = 0

fn main():
    g = Gauge()
    total = tuned(helper(4)) + g.read()
    g.reset()
    println(total + g.read() + 7)
