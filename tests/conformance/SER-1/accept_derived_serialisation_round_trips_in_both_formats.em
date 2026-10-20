#$ test: run-pass
#$ rules: SER-1
#$ stdout: {"$version":2,"name":"ann","pos":{"x":1.5,"y":-2.0},"tags":["a","b"],"best":null,"shape":{"$variant":"Circle","0":0.5},"level":3,"added":false}
#$ stdout: true
#$ stdout: true
#$ stdout: 0 ann
## `[SER-1]` — `@derive(Serialize, Deserialize)` writes a struct as its version and its fields by
## name and an enum value as its variant's name and its payload, through `std.ser.json`'s and
## `std.ser.binary`'s writers and readers; `@ser(skip)` leaves a field out (read back as its
## default), `@ser(rename="…")` names it, `@ser(since=N)` raises the version.

from std.ser import Serialize, Deserialize
import std.ser.json
import std.ser.binary

@derive(Serialize, Deserialize, Eq, Debug)
struct Pos:
    x: float
    y: float

@derive(Serialize, Deserialize, Eq, Debug)
enum Shape:
    Dot
    Circle(float)
    Rect(w: float, h: float)

@derive(Serialize, Deserialize, Eq, Debug)
struct Player:
    name: String
    pos: Pos
    tags: Array[String]
    best: Option[int]
    shape: Shape
    @ser(rename="level")
    rank: int
    @ser(skip)
    cache: int
    @ser(since=2)
    added: bool

fn main():
    p = Player(String.from("ann"), Pos(1.5, -2.0), [String.from("a"), String.from("b")], None, Shape.Circle(0.5), 3, 99, false)
    text = json.to_string(p).unwrap()
    println(text)
    back: Player = json.from_str[Player](text.as_str()).unwrap()
    println(back.pos == p.pos and back.shape == p.shape and back.rank == 3 and back.tags == p.tags)
    bytes = binary.to_bytes(p).unwrap()
    again: Player = binary.from_bytes[Player](bytes.as_span()).unwrap()
    println(again.name == p.name and again.shape == p.shape and again.best == None)
    println(again.cache, again.name)
