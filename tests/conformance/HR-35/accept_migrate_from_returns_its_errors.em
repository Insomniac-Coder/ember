#$ test: run-pass
#$ rules: HR-35, HR-16
#$ stdout: 3
#$ assert-c: contains("struct em_18__reload_Old_Gauge {")
#$ assert-c: contains("em_rl_migrate_5Gauge(void* out, const void* old")
# `migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` replaces field-wise migration for
# its type: `OldSelf` is the previous build's schema as the struct `OldGauge`, which the runtime
# fills from each old instance. A failure is a value it returns, with `?` or `Err(...)`, never a
# panic. The program runs as any other; the function runs only when a reload migrates a `Gauge`.

from std.hot import ReloadError

class Gauge:
    level: int = 3

    fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]:
        if old.level < 0:
            return Err(ReloadError.Migration("Gauge".to_string(), 0, "a negative level".to_string()))
        g = Gauge()
        g.level = old.level
        return Ok(g)

fn main():
    println(Gauge().level)
