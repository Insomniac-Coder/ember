## `std.io` — standard input, and what goes wrong touching the outside world (`[STD-22]`).

from std.fmt import Formatter, FmtError
from std.core import Error as ErrorInterface

## `[STD-22]` — what kind of thing went wrong.
pub enum ErrorKind:
    NotFound
    PermissionDenied
    AlreadyExists
    InvalidData
    Other

## `[STD-22]` — an operation on the outside world that failed: its kind, the C library's code for
## it (`errno`; 0 when the failure is Ember's own, such as text that is not UTF-8), and what
## happened, in words.
pub struct Error:
    pub kind: ErrorKind
    pub code: int
    pub message: String

    ## The failure the C library's code `code` names.
    pub fn from_code(code: int) -> Error:
        kind = match error_kind(code):
            0 => ErrorKind.NotFound
            1 => ErrorKind.PermissionDenied
            2 => ErrorKind.AlreadyExists
            _ => ErrorKind.Other
        message = String()
        error_message(code, message)
        return Error(kind = kind, code = code, message = message)

    ## Data that is not what it should be: text that is not UTF-8, for one.
    pub fn invalid_data(message: str) -> Error:
        return Error(kind = ErrorKind.InvalidData, code = 0, message = message.to_string())

extend Error implements Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(self.message)

## D-749, `[ERR-3]`, `[ERR-8]` — an error type: `?` carries one into an `AnyError` (`fs.read_to_string(path)?`).
extend Error implements ErrorInterface:
    pass

## `[STD-22]` — the process's standard input.
pub struct Stdin:
    pass

## The process's standard input.
pub fn stdin() -> Stdin:
    return Stdin()

extend Stdin:
    ## One line of input without its line ending; `None` at the end of input, and `Err` when the
    ## line is not UTF-8 or reading failed.
    pub fn read_line(self) -> Result[Option[String], Error]:
        bytes: Array[u8] = []
        end = false
        code = stdin_read_line(bytes, end)
        if code != 0:
            return Err(Error.from_code(code))
        if end:
            return Ok(None)
        match bytes.as_span().to_str():
            Ok(text):
                return Ok(Some(text.to_string()))
            Err(_):
                return Err(Error.invalid_data("the line is not UTF-8"))

    ## The lines up to the end of input, each as `read_line` gives it.
    pub fn lines(self) -> Lines:
        return Lines()

## `[STD-22]` — standard input's lines, from `Stdin.lines()`.
pub struct Lines:
    pass

extend Lines implements Iterator:
    type Item = Result[String, Error]

    fn next(mut self) -> Option[Result[String, Error]]:
        match Stdin().read_line():
            Ok(Some(line)):
                return Some(Ok(line))
            Ok(None):
                return None
            Err(error):
                return Some(Err(error))

## The runtime's own (`ember_hir::RuntimeFn`): a call is the runtime's, and these staging bodies
## never run.
fn stdin_read_line(mut out: Array[u8], mut end: bool) -> int:
    return 0

fn error_kind(code: int) -> int:
    return 0

fn error_message(code: int, mut out: String):
    pass
