## `std.fs` — files and directories (`[STD-22]`). Every operation returns a `Result` whose error is
## an `io.Error`, and carries `Io`. A path is UTF-8 text.

import std.io
from std.io import Error

## `[STD-22]` — the bytes of the file at `path`.
## It allocates a new `Array`.
pub fn read(path: StrView) -> Result[Array[u8], Error]:
    bytes: Array[u8] = []
    code = read_raw(path, bytes)
    if code != 0:
        return Err(Error.from_code(code))
    return Ok(bytes)

## The text of the file at `path`; `Err` with `InvalidData` when it is not UTF-8.
## It allocates a new `String`.
pub fn read_to_string(path: StrView) -> Result[String, Error]:
    bytes = read(path)?
    match bytes.as_span().to_str():
        Ok(text):
            return Ok(text.to_string())
        Err(_):
            return Err(Error.invalid_data(f"{path} is not UTF-8"))

## Writes `data` to the file at `path`, making it or replacing what it held.
## It allocates: the operating system is called with a copy of the path or the data.
pub fn write(path: StrView, data: Span[u8]) -> Result[void, Error]:
    return done(write_raw(path, data, false))

## Writes `data` at the end of the file at `path`, making it if it is not there.
## It allocates: the operating system is called with a copy of the path or the data.
pub fn append(path: StrView, data: Span[u8]) -> Result[void, Error]:
    return done(write_raw(path, data, true))

## Whether anything is at `path`.
## It allocates: the operating system is called with a copy of the path or the data.
pub fn exists(path: StrView) -> bool:
    return exists_raw(path)

## Removes the file, or the empty directory, at `path`.
## It allocates: the operating system is called with a copy of the path or the data.
pub fn remove(path: StrView) -> Result[void, Error]:
    return done(remove_raw(path))

## Makes the directory at `path` and each missing directory above it; one already there is kept.
## It allocates: the operating system is called with a copy of the path or the data.
pub fn create_dir_all(path: StrView) -> Result[void, Error]:
    return done(create_dir_all_raw(path))

## The names of what the directory at `path` holds, in order (`.` and `..` left out).
## It allocates a new `Array`.
pub fn read_dir(path: StrView) -> Result[Array[String], Error]:
    names: Array[String] = []
    code = read_dir_raw(path, names)
    if code != 0:
        return Err(Error.from_code(code))
    names.sort()
    return Ok(names)

## `[STD-22]` — what `fs.metadata` says of a path.
pub struct Metadata:
    pub len: int
    pub is_dir: bool
    pub is_file: bool

## What is at `path`: its size in bytes and whether it is a file or a directory.
## It allocates: the operating system is called with a copy of the path or the data.
pub fn metadata(path: StrView) -> Result[Metadata, Error]:
    size = 0
    kind = 0
    code = metadata_raw(path, size, kind)
    if code != 0:
        return Err(Error.from_code(code))
    return Ok(Metadata(len = size, is_dir = kind == 1, is_file = kind == 0))

## `[STD-22]` — an open file. It is move-only, and closes when dropped.
pub struct File:
    handle: int

    ## The file at `path`, open to read.
    ## It allocates: the operating system is called with a copy of the path or the data.
    pub static fn open(path: StrView) -> Result[File, Error]:
        return opened(path, 0)

    ## The file at `path`, made or emptied, open to write.
    ## It allocates: the operating system is called with a copy of the path or the data.
    pub static fn create(path: StrView) -> Result[File, Error]:
        return opened(path, 1)

    ## Reads into `into`; the count read, 0 at the end of the file.
    ## It allocates: the operating system is called with a copy of the path or the data.
    pub fn read(ref mut into: MutSpan[u8]) -> Result[int, Error]:
        got = 0
        code = file_read(this.handle, into, got)
        if code != 0:
            return Err(Error.from_code(code))
        return Ok(got)

    ## Writes all of `data`.
    ## It allocates: the operating system is called with a copy of the path or the data.
    pub fn write(data: Span[u8]) -> Result[void, Error]:
        return done(file_write(this.handle, data))

    ## Writes what the file holds back to the system.
    ## It allocates: the operating system is called with a copy of the path or the data.
    pub fn flush() -> Result[void, Error]:
        return done(file_flush(this.handle))

extend File:
    ## It allocates: the operating system is called with a copy of the path or the data.
    fn drop():
        if this.handle != 0:
            file_close(this.handle)

## `[STD-22]` — a file read through a buffer, a line at a time.
pub struct BufReader:
    file: File
    buffer: Array[u8]
    at: int
    filled: int

    ## A buffered reader of `file`.
    ## It allocates a new `Array`.
    pub static fn new(owns file: File) -> BufReader:
        buffer: Array[u8] = []
        buffer.reserve(8192)
        for _ in 0..8192:
            buffer.push(0)
        return BufReader(file = file, buffer = buffer, at = 0, filled = 0)

    ## One line without its line ending; `None` at the end of the file, and `Err` when the line is
    ## not UTF-8 or reading failed.
    ## It allocates a new `Array`.
    pub fn read_line() -> Result[Option[String], Error]:
        line: Array[u8] = []
        read_any = false
        while true:
            if this.at == this.filled:
                this.filled = this.file.read(this.buffer.as_mut_span())?
                this.at = 0
                if this.filled == 0:
                    break
            byte = this.buffer[this.at]
            this.at += 1
            read_any = true
            if byte == 10:
                break
            line.push(byte)
        if not read_any:
            return Ok(None)
        if line.len() > 0 and line[line.len() - 1] == 13:
            line.pop()
        match line.as_span().to_str():
            Ok(text):
                return Ok(Some(text.to_string()))
            Err(_):
                return Err(Error.invalid_data("the line is not UTF-8"))

## `[STD-22]` — a file written through a buffer, which it flushes when dropped.
pub struct BufWriter:
    file: File
    buffer: Array[u8]

    ## A buffered writer to `file`.
    ## It allocates a new `Array`.
    pub static fn new(owns file: File) -> BufWriter:
        return BufWriter(file = file, buffer = [])

    ## Adds `data` to the buffer, writing the buffer out when it grows past 8 KiB.
    ## It allocates as the `Array` it adds to grows.
    pub fn write(data: Span[u8]) -> Result[void, Error]:
        this.buffer.extend(data)
        if this.buffer.len() >= 8192:
            return this.flush()
        return Ok(())

    ## Writes the buffer out.
    ## It allocates: the operating system is called with a copy of the path or the data.
    pub fn flush() -> Result[void, Error]:
        if this.buffer.len() > 0:
            this.file.write(this.buffer.as_span())?
            this.buffer.clear()
        return this.file.flush()

extend BufWriter:
    ## It allocates: the operating system is called with a copy of the path or the data.
    fn drop():
        _ = this.flush()

## It allocates: the operating system is called with a copy of the path or the data.
fn opened(path: StrView, mode: int) -> Result[File, Error]:
    handle = 0
    code = file_open(path, mode, handle)
    if code != 0:
        return Err(Error.from_code(code))
    return Ok(File(handle = handle))

## It allocates a new `String`.
fn done(code: int) -> Result[void, Error]:
    if code != 0:
        return Err(Error.from_code(code))
    return Ok(())

## The runtime's own (`ember_hir::RuntimeFn`): a call is the runtime's, and these staging bodies
## never run.
fn read_raw(path: StrView, ref mut out: Array[u8]) -> int:
    return 0

fn write_raw(path: StrView, data: Span[u8], append: bool) -> int:
    return 0

fn exists_raw(path: StrView) -> bool:
    return false

fn metadata_raw(path: StrView, ref mut size: int, ref mut kind: int) -> int:
    return 0

fn remove_raw(path: StrView) -> int:
    return 0

fn create_dir_all_raw(path: StrView) -> int:
    return 0

fn read_dir_raw(path: StrView, ref mut names: Array[String]) -> int:
    return 0

fn file_open(path: StrView, mode: int, ref mut handle: int) -> int:
    return 0

fn file_read(handle: int, ref mut into: MutSpan[u8], ref mut got: int) -> int:
    return 0

fn file_write(handle: int, data: Span[u8]) -> int:
    return 0

fn file_flush(handle: int) -> int:
    return 0

fn file_close(handle: int) -> int:
    return 0
