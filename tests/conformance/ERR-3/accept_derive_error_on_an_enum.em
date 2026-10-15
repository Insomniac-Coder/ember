#$ test: run-pass
#$ rules: ERR-3, ERR-2, STR-5
#$ profiles: debug, release, shipping
#$ stdout: line 4: expected `key = value` | ConfigError.Syntax(line=4) | no cause
#$ line 5: `depth` is not a number (5, {braces}) | ConfigError.NotNumber(line=5, key='depth') | no cause
#$ empty | ConfigError.Empty | no cause
#$ reading the file | ConfigError.Io(IoError(code=-9)) | caused by: i/o error -9
#$ ok 7
# `[ERR-3]` — `@derive(Error)` writes `Display` from each variant's
# `@error("…")`, whose `{name}` and `{0}` name its fields (`{{` and `}}` are
# braces), keeps the field-wise `Debug` (`[STR-5]`), and for an `@from`
# variant gives `From` for its field's type, which `?` converts with
# (`[ERR-2]`), and returns that field from `source`.

@derive(Error)
@error("i/o error {code}")
struct IoError:
    code: int

@derive(Error)
enum ConfigError:
    @error("line {line}: expected `key = value`")
    Syntax(line: int)
    @error("line {line}: `{key}` is not a number ({0}, {{braces}})")
    NotNumber(line: int, key: String)
    @error("reading the file")
    @from
    Io(IoError)
    @error("empty")
    Empty

fn read(n: int) -> Result[int, IoError]:
    if n < 0:
        return Err(IoError(n))
    return Ok(n)

fn parse(n: int) -> Result[int, ConfigError]:
    v = read(n)?
    if v == 1:
        return Err(ConfigError.Syntax(line=4))
    if v == 2:
        return Err(ConfigError.NotNumber(line=5, key=String.from("depth")))
    if v == 3:
        return Err(ConfigError.Empty)
    return Ok(v)

fn report(r: Result[int, ConfigError]):
    match r:
        Ok(v):
            println(f"ok {v}")
        Err(e):
            print(f"{e} | {e!r}")
            match e.source():
                Some(c):
                    println(f" | caused by: {c}")
                None:
                    println(" | no cause")

fn main():
    report(parse(1))
    report(parse(2))
    report(parse(3))
    report(parse(-9))
    report(parse(7))
