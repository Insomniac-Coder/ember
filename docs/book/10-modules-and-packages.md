# 10. Modules and packages

## Importing

Each file is a module. `import` makes another module's public items reachable through its name;
`from … import` names items directly. The standard library's modules are under `std`, which may be
left out:

```ember
import math
from std.collections import Map

fn main():
    println(math.sqrt(16.0), math.PI > 3.14)
    counts: Map[str, int] = {}
    counts["a"] = 1
    println(counts)
```

```output
4.0 true
{'a': 1}
```

A module's items are private to it unless marked `pub`:

```ember,fragment
pub struct Vector:
    pub x: float
    pub y: float

pub fn length(v: Vector) -> float:
    return (v.x * v.x + v.y * v.y).sqrt()

fn helper() -> int:
    return 1
```

Saved as `geometry/vector.em` in a package's `src/`, another file reaches it with
`import geometry.vector` (then `vector.length(…)`) or `from geometry.vector import length`.

## The standard library

Among its modules: `std.collections` (`Array`, `Map`, `Set` and more), `std.string`, `std.math`,
`std.fs` (files), `std.io`, `std.env` (arguments and environment), `std.process`, `std.time`,
`std.random`, `std.thread` and `std.sync`, `std.fmt`. `ember doc` on a file shows each function's
contract: how it takes each parameter, what its result borrows, whether it allocates.

```ember
import std.fs

fn main():
    path = "guide-example.txt"
    match fs.write(path, "one\ntwo\n".as_bytes()):
        Ok(_):
            pass
        Err(e):
            println("cannot write:", e)
    match fs.read_to_string(path):
        Ok(text):
            for line in text.lines():
                println(line.to_upper())
        Err(e):
            println("cannot read:", e)
    _ = fs.remove(path)
```

```output
ONE
TWO
```

## A package

`ember new name` makes a package: a folder with `ember.toml` and `src/main.em` (or `src/lib.em` for
a library, `ember new --lib`). The manifest names the package and what it depends on:

```toml
[package]
name = "inventory"
version = "0.1.0"

[dependencies]
geometry = { path = "../geometry" }
```

`ember build` builds the whole package; `ember.lock` records the versions it resolved.
