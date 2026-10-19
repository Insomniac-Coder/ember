## `std.fs` — files and directories (`[STD-22]`). Every operation returns a `Result` whose error is
## an `io.Error`, and carries `Io`. A path is UTF-8 text.

import std.io
from std.io import Error

## `[STD-22]` — the bytes of the file at `path`.
pub fn read(path: str) -> Result[Array[u8], Error]:
    bytes: Array[u8] = []
    code = read_raw(path, bytes)
    if code != 0:
        return Err(Error.from_code(code))
    return Ok(bytes)

## The text of the file at `path`; `Err` with `InvalidData` when it is not UTF-8.
pub fn read_to_string(path: str) -> Result[String, Error]:
    bytes = read(path)?
    match bytes.as_span().to_str():
        Ok(text):
            return Ok(text.to_string())
        Err(_):
            return Err(Error.invalid_data(f"{path} is not UTF-8"))

## Writes `data` to the file at `path`, making it or replacing what it held.
pub fn write(path: str, data: Span[u8]) -> Result[void, Error]:
    return done(write_raw(path, data, false))

## Writes `data` at the end of the file at `path`, making it if it is not there.
pub fn append(path: str, data: Span[u8]) -> Result[void, Error]:
    return done(write_raw(path, data, true))

## Whether anything is at `path`.
pub fn exists(path: str) -> bool:
    return exists_raw(path)

## Removes the file, or the empty directory, at `path`.
pub fn remove(path: str) -> Result[void, Error]:
    return done(remove_raw(path))

## Makes the directory at `path` and each missing directory above it; one already there is kept.
pub fn create_dir_all(path: str) -> Result[void, Error]:
    return done(create_dir_all_raw(path))

## The names of what the directory at `path` holds, in order (`.` and `..` left out).
pub fn read_dir(path: str) -> Result[Array[String], Error]:
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
pub fn metadata(path: str) -> Result[Metadata, Error]:
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
    pub fn open(path: str) -> Result[File, Error]:
        return opened(path, 0)

    ## The file at `path`, made or emptied, open to write.
    pub fn create(path: str) -> Result[File, Error]:
        return opened(path, 1)

    ## Reads into `into`; the count read, 0 at the end of the file.
    pub fn read(mut self, mut into: MutSpan[u8]) -> Result[int, Error]:
        got = 0
        code = file_read(self.handle, into, got)
        if code != 0:
            return Err(Error.from_code(code))
        return Ok(got)

    ## Writes all of `data`.
    pub fn write(mut self, data: Span[u8]) -> Result[void, Error]:
        return done(file_write(self.handle, data))

    ## Writes what the file holds back to the system.
    pub fn flush(mut self) -> Result[void, Error]:
        return done(file_flush(self.handle))

extend File:
    fn drop(mut self):
        if self.handle != 0:
            file_close(self.handle)

## `[STD-22]` — a file read through a buffer, a line at a time.
pub struct BufReader:
    file: File
    buffer: Array[u8]
    at: int
    filled: int

    ## A buffered reader of `file`.
    pub fn new(owned file: File) -> BufReader:
        buffer: Array[u8] = []
        buffer.reserve(8192)
        for _ in 0..8192:
            buffer.push(0)
        return BufReader(file = file, buffer = buffer, at = 0, filled = 0)

    ## One line without its line ending; `None` at the end of the file, and `Err` when the line is
    ## not UTF-8 or reading failed.
    pub fn read_line(mut self) -> Result[Option[String], Error]:
        line: Array[u8] = []
        read_any = false
        while true:
            if self.at == self.filled:
                self.filled = self.file.read(self.buffer.as_mut_span())?
                self.at = 0
                if self.filled == 0:
                    break
            byte = self.buffer[self.at]
            self.at += 1
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
    pub fn new(owned file: File) -> BufWriter:
        return BufWriter(file = file, buffer = [])

    ## Adds `data` to the buffer, writing the buffer out when it grows past 8 KiB.
    pub fn write(mut self, data: Span[u8]) -> Result[void, Error]:
        self.buffer.extend(data)
        if self.buffer.len() >= 8192:
            return self.flush()
        return Ok(())

    ## Writes the buffer out.
    pub fn flush(mut self) -> Result[void, Error]:
        if self.buffer.len() > 0:
            self.file.write(self.buffer.as_span())?
            self.buffer.clear()
        return self.file.flush()

extend BufWriter:
    fn drop(mut self):
        _ = self.flush()

fn opened(path: str, mode: int) -> Result[File, Error]:
    handle = 0
    code = file_open(path, mode, handle)
    if code != 0:
        return Err(Error.from_code(code))
    return Ok(File(handle = handle))

fn done(code: int) -> Result[void, Error]:
    if code != 0:
        return Err(Error.from_code(code))
    return Ok(())

## The runtime's own (`ember_hir::RuntimeFn`): a call is the runtime's, and these staging bodies
## never run.
fn read_raw(path: str, mut out: Array[u8]) -> int:
    return 0

fn write_raw(path: str, data: Span[u8], append: bool) -> int:
    return 0

fn exists_raw(path: str) -> bool:
    return false

fn metadata_raw(path: str, mut size: int, mut kind: int) -> int:
    return 0

fn remove_raw(path: str) -> int:
    return 0

fn create_dir_all_raw(path: str) -> int:
    return 0

fn read_dir_raw(path: str, mut names: Array[String]) -> int:
    return 0

fn file_open(path: str, mode: int, mut handle: int) -> int:
    return 0

fn file_read(handle: int, mut into: MutSpan[u8], mut got: int) -> int:
    return 0

fn file_write(handle: int, data: Span[u8]) -> int:
    return 0

fn file_flush(handle: int) -> int:
    return 0

fn file_close(handle: int) -> int:
    return 0
