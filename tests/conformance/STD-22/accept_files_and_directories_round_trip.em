#$ test: run-pass
#$ rules: STD-22
#$ stdout: hello
#$ world
#$ again
#$
#$ ['a', 'one.txt']
#$ 18 true false
#$ x
#$ y
#$ NotFound true
#$ false
# `[STD-22]` — `std.fs`: every operation returns a `Result` whose error is an `io.Error`. A
# directory tree is made, a file written, appended to and read back as bytes and as text,
# listed and measured; a `BufWriter` writes through a `File` and a `BufReader` reads it back a
# line at a time (`\r\n` ends a line too); a missing file is `NotFound`. The `File`s close when
# dropped, here at the end of `copy_lines`, before the files are removed.

import std.env
import std.fs
import std.io
import std.random
from std.fs import File, BufReader, BufWriter

fn scratch() -> String:
    base = "/tmp".to_string()
    match env.var("TEMP"):
        Some(dir):
            base = dir
        None:
            match env.var("TMPDIR"):
                Some(dir):
                    base = dir
                None:
                    pass
    rng = random.Rng.from_entropy()
    return f"{base}/ember_std22_{rng.next_u64()}"

fn copy_lines(path: str) -> Result[void, io.Error]:
    out = BufWriter.new(File.create(path)?)
    out.write("x\r\ny\n".as_bytes())?
    out.flush()?
    reader = BufReader.new(File.open(path)?)
    while true:
        match reader.read_line()?:
            Some(line):
                println(line)
            None:
                break
    return Ok(())

fn main() -> Result[void, io.Error]:
    root = scratch()
    fs.create_dir_all(f"{root}/a/b")?
    one = f"{root}/one.txt"
    fs.write(one, "hello\nworld\n".as_bytes())?
    fs.append(one, "again\n".as_bytes())?
    println(fs.read_to_string(one)?)
    println(fs.read_dir(root)?)
    meta = fs.metadata(one)?
    bytes = fs.read(one)?
    println(meta.len, meta.is_file and bytes.len() == 18, meta.is_dir)
    two = f"{root}/two.txt"
    copy_lines(two)?
    match fs.read(f"{root}/missing.txt"):
        Ok(_):
            println("found")
        Err(error):
            match error.kind:
                io.ErrorKind.NotFound:
                    println("NotFound", error.message.len() > 0)
                _:
                    println("another kind")
    fs.remove(two)?
    fs.remove(one)?
    fs.remove(f"{root}/a/b")?
    fs.remove(f"{root}/a")?
    fs.remove(root)?
    println(fs.exists(root))
    return Ok(())
