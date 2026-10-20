#$ test: run-pass
#$ rules: SER-2, RNG-10a, SER-1
#$ stdout: Err(SerError.MissingField('hp'))
#$ stdout: Err(SerError.UnknownVariant('Square'))
#$ stdout: Err(SerError.OutOfRange('hp'))
#$ stdout: 50 7 0
#$ stdout: Err(SerError.OutOfRange('hp'))
## `[SER-2]`, `[RNG-10a]` — deserialising never produces an invalid value: a field the input lacks
## is `MissingField` unless it is `@ser(default)` or `@ser(since=N)`, which take their default; a
## variant the enum lacks is `UnknownVariant`; a range-typed field is checked with `checked`, and a
## value out of range is `OutOfRange`. A field the type does not know is skipped, so older and newer
## versions of a type read each other's data.

from std.ser import Serialize, Deserialize, SerError
import std.ser.json
import std.ser.binary

type Health = int in 0..=100

@derive(Serialize, Deserialize, Debug)
enum Kind:
    Round
    Long(len: int)

@derive(Serialize, Deserialize, Debug)
struct Unit:
    hp: Health
    @ser(default)
    armour: int
    @ser(since=3)
    level: int

@derive(Serialize, Deserialize, Debug)
struct Wide:
    hp: int
    extra: Array[int]

fn main():
    missing: Result[Unit, SerError] = json.from_str[Unit]("{\"armour\":1}")
    println(missing.map(fn(u) => u.armour))
    unknown: Result[Kind, SerError] = json.from_str[Kind]("{\"$variant\":\"Square\"}")
    println(unknown.map(fn(k) => 0))
    high: Result[Unit, SerError] = json.from_str[Unit]("{\"hp\":101}")
    println(high.map(fn(u) => u.armour))
    newer = json.to_string(Wide(50, [1, 2])).unwrap()
    read: Unit = json.from_str[Unit](newer.as_str()).unwrap()
    w: Wide = json.from_str[Wide]("{\"hp\":7,\"extra\":[]}").unwrap()
    println(read.hp, w.hp, read.level)
    bytes = binary.to_bytes(Wide(-3, [])).unwrap()
    println(binary.from_bytes[Unit](bytes.as_span()).map(fn(u) => u.armour))
