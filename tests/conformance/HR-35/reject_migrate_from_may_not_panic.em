#$ test: compile-fail
#$ rules: HR-35, HR-16
#$ help: use `?` to return the error
#$ help: return `Err(ReloadError.Migration(type_name, instance, message))` instead
#$ help: test it with `if` and return `Err(ReloadError.Migration(…))`
# `migrate_from` cannot abort the process: an explicit `panic`, `unwrap`, `expect` or `assert` in
# it is `E2225`, whose help names the fallible form. The words in a string or a comment are not
# calls: panic("no") here is text.

from std.hot import ReloadError

class Gauge:
    level: int = 0

    fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]:
        parsed = "4".parse[int]().unwrap()  #$ error[E2225]: `migrate_from` may not panic: `unwrap` would end the process
        checked = "5".parse[int]().expect("a number")  #$ error[E2225]: `expect` would end the process
        assert(old.level >= 0, "never negative")  #$ error[E2225]: `assert` would end the process
        if old.level > 100:
            panic("too high")  #$ error[E2225]: `panic` would end the process
        g = Gauge()
        g.level = old.level + parsed + checked
        return Ok(g)

fn main():
    println(Gauge().level)
