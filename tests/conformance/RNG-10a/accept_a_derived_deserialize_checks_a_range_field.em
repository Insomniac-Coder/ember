#$ test: run-pass
#$ rules: RNG-10a
#$ stdout: Err(SerError.OutOfRange('hp'))
#$ stdout: 50
## `[RNG-10a]` — a derived `Deserialize` checks every range-typed field with `checked` and maps
## failure to `SerError`: a value out of range is `OutOfRange`.

from std.ser import Serialize, Deserialize, SerError
import std.ser.json

type Health = int in 0..=100

@derive(Serialize, Deserialize, Debug)
struct Unit:
    hp: Health

fn main():
    bad: Result[Unit, SerError] = json.from_str[Unit]("{\"hp\":200}")
    println(bad.map(owns fn(u) => u.hp))
    good: Result[Unit, SerError] = json.from_str[Unit]("{\"hp\":50}")
    println(good.map(owns fn(u) => u.hp).unwrap_or(0))
