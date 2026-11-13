//! Lane b's checks of the C interface (`[FFI-26]`): the unsafe form of the export attributes and the
//! C names an export may not take.

use super::*;

/// `[FFI-26]` — the functions of the C standard library (C11's headers; each `<math.h>` and
/// `<complex.h>` function with its `f` and `l` forms, `MATH` and `COMPLEX` below): an export under
/// one of these names would replace it for the whole process. The same list on every target.
const C_LIBRARY: &str = "isalnum isalpha isblank iscntrl isdigit isgraph islower isprint ispunct isspace isupper \
    isxdigit tolower toupper feclearexcept fegetexceptflag feraiseexcept fesetexceptflag fetestexcept fegetround \
    fesetround fegetenv feholdexcept fesetenv feupdateenv imaxabs imaxdiv strtoimax strtoumax wcstoimax wcstoumax \
    setlocale localeconv setjmp longjmp signal raise atomic_thread_fence atomic_signal_fence atomic_flag_test_and_set \
    atomic_flag_test_and_set_explicit atomic_flag_clear atomic_flag_clear_explicit remove rename tmpfile tmpnam fclose \
    fflush fopen freopen setbuf setvbuf fprintf fscanf printf scanf snprintf sprintf sscanf vfprintf vfscanf vprintf \
    vscanf vsnprintf vsprintf vsscanf fgetc fgets fputc fputs getc getchar gets putc putchar puts ungetc fread fwrite \
    fgetpos fseek fsetpos ftell rewind clearerr feof ferror perror atof atoi atol atoll strtod strtof strtold strtol \
    strtoll strtoul strtoull rand srand aligned_alloc calloc free malloc realloc abort atexit at_quick_exit exit _Exit \
    getenv quick_exit system bsearch qsort abs labs llabs div ldiv lldiv mblen mbtowc wctomb mbstowcs wcstombs memcpy \
    memmove strcpy strncpy strcat strncat memcmp strcmp strcoll strncmp strxfrm memchr strchr strcspn strpbrk strrchr \
    strspn strstr strtok memset strerror strlen call_once cnd_broadcast cnd_destroy cnd_init cnd_signal cnd_timedwait \
    cnd_wait mtx_destroy mtx_init mtx_lock mtx_timedlock mtx_trylock mtx_unlock thrd_create thrd_current thrd_detach \
    thrd_equal thrd_exit thrd_join thrd_sleep thrd_yield tss_create tss_delete tss_get tss_set clock difftime mktime \
    time timespec_get asctime ctime gmtime localtime strftime mbrtoc16 c16rtomb mbrtoc32 c32rtomb fwprintf fwscanf \
    swprintf swscanf vfwprintf vfwscanf vswprintf vswscanf vwprintf vwscanf wprintf wscanf fgetwc fgetws fputwc \
    fputws fwide getwc getwchar putwc putwchar ungetwc wcstod wcstof wcstold wcstol wcstoll wcstoul wcstoull wcscpy \
    wcsncpy wmemcpy wmemmove wcscat wcsncat wcscmp wcscoll wcsncmp wcsxfrm wmemcmp wcschr wcscspn wcspbrk wcsrchr \
    wcsspn wcsstr wcstok wmemchr wcslen wmemset wcsftime btowc wctob mbsinit mbrlen mbrtowc wcrtomb mbsrtowcs \
    wcsrtombs iswalnum iswalpha iswblank iswcntrl iswdigit iswgraph iswlower iswprint iswpunct iswspace iswupper \
    iswxdigit iswctype wctype towlower towupper towctrans wctrans";
const MATH: &str = "acos asin atan atan2 cos sin tan acosh asinh atanh cosh sinh tanh exp exp2 expm1 frexp ilogb \
    ldexp log log10 log1p log2 logb modf scalbn scalbln cbrt fabs hypot pow sqrt erf erfc lgamma tgamma ceil floor \
    nearbyint rint lrint llrint round lround llround trunc fmod remainder remquo copysign nan nextafter nexttoward fdim \
    fmax fmin fma";
const COMPLEX: &str = "cacos casin catan ccos csin ctan cacosh casinh catanh ccosh csinh ctanh cexp clog cabs cpow \
    csqrt carg cimag conj cproj creal";

/// Whether `symbol` is a function of the C standard library.
fn c_library_function(symbol: &str) -> bool {
    C_LIBRARY.split_whitespace().any(|name| name == symbol)
        || MATH.split_whitespace().chain(COMPLEX.split_whitespace()).any(|name| {
            symbol.strip_prefix(name).is_some_and(|suffix| suffix.is_empty() || suffix == "f" || suffix == "l")
        })
}

impl Checker<'_> {
    /// `[FFI-26]` — an export names a C symbol the whole process shares, which the compiler cannot
    /// see is free: the attribute is written in its unsafe form, `@unsafe(export("name"))`, and the
    /// plain one is `E3100`; no other attribute takes that form (`[TIER-1]`). `item` names what the
    /// attribute gives a name to, as the message says it.
    pub(crate) fn check_unsafe_attribute_form(&mut self, attr: &ast::Attribute, item: &str) {
        let name = attr.path.iter().map(|segment| segment.name.as_str()).collect::<Vec<_>>().join(".");
        let export = matches!(name.as_str(), "export" | "export_table");
        if attr.written_unsafe && !export {
            self.sink.emit(
                Diagnostic::error(codes::E0104, attr.span, format!("`@{name}` has no unsafe form: only `@unsafe(export(…))` and `@unsafe(export_table(…))` are written so"))
                    .help(format!("write `@{name}(…)` [FFI-26], [TIER-1]")),
            );
            return;
        }
        if attr.written_unsafe || !export {
            return;
        }
        let written = self.written_line(attr.span).unwrap_or_else(|| format!("@{name}(…)"));
        let inner = written.trim_start_matches('@');
        let gives = if name == "export" { format!("gives `{item}` a C name") } else { format!("gives `{item}`'s table a C name") };
        self.sink.emit(
            Diagnostic::error(
                codes::E3100,
                attr.span,
                format!("`{written}` {gives} the whole process shares, and the compiler cannot see whether the host or a library already defines it; to vouch for it, write `@unsafe({inner})` with a `# SAFETY:` note above it"),
            )
            .note("a C symbol is one name for the whole process [FFI-26]"),
        );
    }

    /// `[FFI-26]` — a C name the compiler knows is taken is refused even in the unsafe form
    /// (`E0104`): `main`, a name beginning with the runtime's prefix (`[RT-5]`), the module-protocol
    /// names of `[FFI-28]`, and the functions of the C standard library. `function` is the Ember
    /// function exported under `symbol`.
    pub(crate) fn check_export_name_taken(&mut self, symbol: &str, function: Symbol, span: Span) {
        let prefix = ember_branding::RUNTIME_PREFIX;
        let protocol = symbol == format!("{prefix}module_init")
            || symbol == format!("{prefix}module_shutdown")
            || symbol.strip_prefix(&format!("{prefix}module_protocol_v")).is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        // `main` is refused where the entry point is known (`check_program`): the export is recorded.
        if symbol == "main" {
            self.exports_main = true;
            return;
        }
        let message = if protocol {
            format!("`{symbol}` is a name of the module protocol, which the runtime defines for every module (`[FFI-28]`); exporting `{function}` under it would replace it; choose another name")
        } else if symbol.starts_with(prefix) {
            format!("`{symbol}` begins with `{prefix}`, the runtime's prefix (`[RT-5]`), and exporting `{function}` under it could replace a runtime function for the whole process; choose another name")
        } else if c_library_function(symbol) {
            let allocates = matches!(symbol, "malloc" | "calloc" | "realloc" | "free" | "aligned_alloc");
            let included = if allocates { ", the runtime's own allocations included" } else { "" };
            format!("`{symbol}` is a function of the C standard library, and exporting `{function}` under its name would replace it for the whole process{included}; choose another name")
        } else {
            return;
        };
        self.sink.emit(Diagnostic::error(codes::E0104, span, message).note("a C name the compiler knows is taken is refused even in the unsafe form [FFI-26]"));
    }
}

#[cfg(test)]
mod tests {
    use super::c_library_function;

    /// `[FFI-26]` — the C library's names, with the `f` and `l` forms of the maths.
    #[test]
    fn the_c_library_list_names_each_form() {
        for name in ["realloc", "malloc", "printf", "sqrt", "sqrtf", "sqrtl", "cabsf", "thrd_create", "_Exit"] {
            assert!(c_library_function(name), "{name}");
        }
        for name in ["game_on_update", "sqrtx", "reallocf", "score"] {
            assert!(!c_library_function(name), "{name}");
        }
    }
}

/// `[FFI-11]` — the contract words a help lists: `unsafe` is none (a declaration leaves its calls
/// unsafe by being written `unsafe fn`), and `TODO` marks a fact not stated yet.
pub(crate) fn contract_words_shown() -> String {
    FFI_CONTRACT_WORDS.iter().filter(|word| !matches!(**word, "unsafe" | "TODO")).copied().collect::<Vec<_>>().join(", ")
}

impl Checker<'_> {
    /// `[FFI-11]`, `[FFI-52]` — `@ffi(no_callbacks)`: the function never calls back into Ember. On one
    /// whose own contract takes a callback, or that takes a function pointer, it is `E5069`.
    pub(crate) fn check_no_callbacks(&mut self, attrs: &[ast::Attribute], decl: &ast::FnDecl, span: Span) {
        let name = decl.name.name;
        let callback = attrs
            .iter()
            .filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ffi"))
            .flat_map(|attr| attr.args.iter())
            .find_map(|arg| match arg {
                ast::AttrArg::Expr(expr) if ffi_callback_param_arg(arg).is_some() => Some(self.written_line(expr.span).unwrap_or_default()),
                _ => None,
            });
        let pointer = decl.params.iter().find_map(|param| match &param.kind {
            ast::ParamKind::Named { name, ty } if matches!(&ty.kind, ast::TypeKind::Fn { abi: Some(_), .. }) => Some(name.name),
            _ => None,
        });
        let takes = match (callback, pointer) {
            (Some(written), _) => format!("takes a callback (`{written}`)"),
            (None, Some(parameter)) => format!("takes a function pointer (`{parameter}`)"),
            (None, None) => return,
        };
        self.sink.emit(
            Diagnostic::error(codes::E5069, span, format!("`{name}` {takes}, so it may call back into Ember; remove `no_callbacks` from its `@ffi(...)`"))
                .note("`no_callbacks` asserts that a call of the function runs no Ember code [FFI-52]"),
        );
    }
}

impl Checker<'_> {
    /// `[FFI-36]` — `adopt` of an imported C++ class with no public destructor: nothing Ember can call
    /// ends one, so the overlay names the function that does (`E5002`). The function suggested is the
    /// header's one that takes one `*mut T` of it, where there is exactly one.
    pub(crate) fn refuse_no_public_destructor(&mut self, module: usize, pointee: Ty, span: Span) {
        let shown = self.types.foreign_object_shown(pointee);
        let prefix = format!("{}.", self.prefixes[module]);
        let mut candidates: Vec<String> = self
            .fn_ids
            .iter()
            .filter(|(path, def)| {
                path.as_str().starts_with(&prefix) && self.frees_one(**def, pointee) && self.signatures[def.0 as usize].ret == self.common.void
            })
            .map(|(path, _)| path.as_str().rsplit('.').next().unwrap_or_default().to_string())
            // The importer's own thunks are no function of the header.
            .filter(|name| !name.starts_with(ember_branding::RUNTIME_PREFIX))
            .collect();
        candidates.sort();
        candidates.dedup();
        let destructor = match candidates.as_slice() {
            [one] => one.clone(),
            _ => "f".to_string(),
        };
        self.sink.emit(
            Diagnostic::error(
                codes::E5002,
                span,
                format!("`{shown}` has no public destructor, so nothing Ember can call ends an adopted one; name the function that does in the overlay: `@ffi(destroys({destructor}))` above `type {shown}`"),
            )
            .note("an adopted object is ended by its type's one destructor [FFI-36], [FFI-36a]"),
        );
    }
}
