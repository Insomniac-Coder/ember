//! The runtime's entry points the IR calls (`[RT-5]`: every name under `EMBER_SYMBOL_PREFIX`),
//! each with its C prototype as `ember_rt.h` declares it. The IR passes their arguments by the
//! platform C ABI (`crate::abi`), so a prototype here must be the header's exactly: the unit test
//! below reads the header and compares every one.

use crate::abi::{Scalar, Shape};

/// A C type of a runtime prototype.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CTy {
    Void,
    Scalar(Scalar),
    /// One of the runtime's structs, by its name without the prefix (`str`, `loc`).
    Struct(&'static str),
}

impl CTy {
    /// The ABI's view of it; `None` for `void`.
    pub fn shape(self) -> Option<Shape> {
        let word = Scalar::Int { bytes: 8, signed: false };
        let half = Scalar::Int { bytes: 4, signed: false };
        let agg = |size, align, leaves: &[(u64, Scalar)]| Shape::Aggregate { size, align, leaves: leaves.to_vec(), unaligned: false };
        Some(match self {
            CTy::Void => return None,
            CTy::Scalar(scalar) => Shape::Scalar(scalar),
            CTy::Struct("str" | "span" | "mutspan") => agg(16, 8, &[(0, Scalar::Ptr), (8, word)]),
            CTy::Struct("dyn") => agg(16, 8, &[(0, Scalar::Ptr), (8, Scalar::Ptr)]),
            CTy::Struct("vec") => agg(24, 8, &[(0, Scalar::Ptr), (8, word), (16, word)]),
            CTy::Struct("loc") => agg(16, 8, &[(0, Scalar::Ptr), (8, half), (12, half)]),
            CTy::Struct("i128") => Shape::Scalar(Scalar::Int { bytes: 16, signed: true }),
            CTy::Struct("u128") => Shape::Scalar(Scalar::Int { bytes: 16, signed: false }),
            CTy::Struct("i256" | "u256") => agg(32, 16, &[(0, word), (8, word), (16, word), (24, word)]),
            // `{uint32 fill; char align, sign; bool alternate, zero; uint32 width; char grouping;
            // int32 precision; char kind}`.
            CTy::Struct("fmt_spec") => {
                let byte = Scalar::Int { bytes: 1, signed: true };
                agg(24, 4, &[(0, half), (4, byte), (5, byte), (6, Scalar::Bool), (7, Scalar::Bool), (8, half), (12, byte), (16, half), (20, byte)])
            }
            CTy::Struct(other) => unreachable!("no shape for the runtime struct `{other}`"),
        })
    }
}

/// Parse one C type as the table writes it: `$name` is the runtime's `ember_name`.
pub fn c_type(text: &'static str) -> CTy {
    let text = text.trim();
    // A pointer, a function pointer among them (`void (*)(void)`).
    if text.ends_with('*') || text.contains("(*") {
        return CTy::Scalar(Scalar::Ptr);
    }
    let int = |bytes, signed| CTy::Scalar(Scalar::Int { bytes, signed });
    match text {
        "void" => CTy::Void,
        "bool" => CTy::Scalar(Scalar::Bool),
        "int8_t" => int(1, true),
        "int16_t" => int(2, true),
        "int32_t" | "int" => int(4, true),
        "int64_t" | "ptrdiff_t" => int(8, true),
        "uint8_t" => int(1, false),
        "uint16_t" => int(2, false),
        "uint32_t" => int(4, false),
        "uint64_t" | "size_t" | "uintptr_t" => int(8, false),
        "float" => CTy::Scalar(Scalar::F32),
        "double" => CTy::Scalar(Scalar::F64),
        other => match other.strip_prefix('$') {
            Some(name) => CTy::Struct(name),
            None => panic!("the runtime table names an unknown C type `{other}`"),
        },
    }
}

/// One entry point: its name without the prefix, its result and parameters as C writes them,
/// whether it never returns (a panic: `cold` too, `[CG-C-5]`), and whether the header defines it
/// inline (`static inline`, or a macro): the IR reaches such a one through the module's shim, a
/// C unit of one-line wrappers that call it exactly as the header defines it (`shim_source`),
/// which link-time optimisation inlines back into the IR.
pub struct RtFn {
    pub name: &'static str,
    pub ret: &'static str,
    pub params: &'static [&'static str],
    pub noreturn: bool,
    pub inline: bool,
}

const fn f(name: &'static str, ret: &'static str, params: &'static [&'static str]) -> RtFn {
    RtFn { name, ret, params, noreturn: false, inline: false }
}

const fn panic_fn(name: &'static str, params: &'static [&'static str]) -> RtFn {
    RtFn { name, ret: "void", params, noreturn: true, inline: false }
}

/// A function the header defines inline.
const fn inl(name: &'static str, ret: &'static str, params: &'static [&'static str]) -> RtFn {
    RtFn { name, ret, params, noreturn: false, inline: true }
}

/// The symbol the IR calls for `entry`: the runtime's own, or its shim's.
pub fn symbol(entry: &RtFn) -> String {
    if entry.inline {
        ember_branding::runtime(&format!("ll_{}", entry.name))
    } else {
        ember_branding::runtime(entry.name)
    }
}

/// The C unit of the shims of `used` (the inline entries the module calls).
pub fn shim_source(used: &std::collections::BTreeSet<&'static str>) -> String {
    let spell = |text: &str| text.replace('$', ember_branding::RUNTIME_PREFIX);
    let mut out = String::new();
    out.push_str("/* Generated by the Ember compiler. Do not edit. */\n");
    out.push_str("/* [CG-LL-1] the runtime's inline functions, for the program's LLVM IR to call. */\n");
    out.push_str(&format!("#include \"{}\"\n\n", ember_branding::runtime_header()));
    for name in used {
        let entry = find(name);
        let params: Vec<String> = entry.params.iter().enumerate().map(|(index, ty)| format!("{} a{index}", spell(ty))).collect();
        let arguments: Vec<String> = (0..entry.params.len()).map(|index| format!("a{index}")).collect();
        let params = if params.is_empty() { "void".to_string() } else { params.join(", ") };
        let call = format!("{}({})", ember_branding::runtime(entry.name), arguments.join(", "));
        let body = if entry.ret == "void" { format!("{call};") } else { format!("return {call};") };
        out.push_str(&format!("{} {}({params}) {{ {body} }}\n", spell(entry.ret), symbol(entry)));
    }
    out
}

pub const FNS: &[RtFn] = &[
    // -- panics ([RT-4], VI.6)
    panic_fn("panic", &["const char*", "size_t", "$loc"]),
    panic_fn("panic_bounds", &["size_t", "size_t", "$loc"]),
    panic_fn("panic_overflow", &["const char*", "$loc"]),
    panic_fn("panic_div_zero", &["$loc"]),
    panic_fn("panic_refcell", &["const char*", "uint32_t", "$loc"]),
    // -- printing ([STD-9])
    f("print_str", "void", &["$str"]),
    f("println_str", "void", &["$str"]),
    f("eprint_str", "void", &["$str"]),
    f("eprintln_str", "void", &["$str"]),
    f("print_i64", "void", &["int64_t"]),
    f("eprint_i64", "void", &["int64_t"]),
    f("print_u64", "void", &["uint64_t"]),
    f("eprint_u64", "void", &["uint64_t"]),
    f("print_i128", "void", &["$i128"]),
    f("eprint_i128", "void", &["$i128"]),
    f("print_u128", "void", &["$u128"]),
    f("eprint_u128", "void", &["$u128"]),
    f("print_i256", "void", &["$i256"]),
    f("eprint_i256", "void", &["$i256"]),
    f("print_u256", "void", &["$u256"]),
    f("eprint_u256", "void", &["$u256"]),
    f("print_f16", "void", &["uint16_t"]),
    f("eprint_f16", "void", &["uint16_t"]),
    f("print_f32", "void", &["float"]),
    f("eprint_f32", "void", &["float"]),
    f("print_f64", "void", &["double"]),
    f("eprint_f64", "void", &["double"]),
    f("print_bool", "void", &["bool"]),
    f("eprint_bool", "void", &["bool"]),
    f("print_char", "void", &["uint32_t"]),
    f("eprint_char", "void", &["uint32_t"]),
    // -- class objects and counts ([OBJ-1], [RC-4], [RT-10])
    f("obj_new", "$obj_header*", &["const $type_info*"]),
    f("obj_new_frame", "$obj_header*", &["void*", "const $type_info*"]),
    f("obj_retain", "void", &["$obj_header*"]),
    f("obj_release", "void", &["$obj_header*"]),
    panic_fn("obj_retain_failed", &["$obj_header*", "uint32_t"]),
    f("obj_release_ended", "void", &["$obj_header*", "uint32_t"]),
    f("obj_check_thread", "void", &["const $obj_header*"]),
    f("weak_retain", "void", &["$obj_header*"]),
    f("weak_release", "void", &["$obj_header*"]),
    f("downcast", "void*", &["$obj_header*", "const $type_info*"]),
    f("itable_lookup_slow", "const void*", &["const $type_info*", "const void*"]),
    f("box_new_copy", "void*", &["size_t", "size_t", "const void*"]),
    f("free", "void", &["void*", "size_t", "size_t"]),
    // -- lists ([HEAP-9], ADR-141)
    f("vec_free", "void", &["$vec*", "size_t"]),
    f("vec_free_aligned", "void", &["$vec*", "size_t", "size_t"]),
    f("stack_find", "void", &[]),
    f("vec_grown", "$vec", &["$vec", "size_t"]),
    f("vec_reserve_aligned", "void", &["$vec*", "size_t", "size_t", "size_t"]),
    f("vec_from_elems", "$vec", &["size_t", "const void*", "size_t"]),
    f("vec_from_elems_aligned", "$vec", &["size_t", "size_t", "const void*", "size_t"]),
    inl("vec_reserve_more_inline", "void", &["$vec*", "size_t", "size_t"]),
    inl("vec_reserve_more_aligned", "void", &["$vec*", "size_t", "size_t", "size_t"]),
    f("vec_reserve_hint", "void", &["$vec*", "size_t", "uint64_t", "size_t"]),
    f("vec_reserve_hint_aligned", "void", &["$vec*", "size_t", "size_t", "uint64_t", "size_t"]),
    f("vec_swap", "void", &["$vec*", "size_t", "size_t", "size_t"]),
    f("vec_insert", "void", &["$vec*", "size_t", "size_t", "const void*"]),
    f("vec_insert_aligned", "void", &["$vec*", "size_t", "size_t", "size_t", "const void*"]),
    f("vec_reverse", "void", &["$vec*", "size_t"]),
    f("vec_extend", "void", &["$vec*", "const void*", "size_t"]),
    f("vec_push_char", "void", &["$vec*", "uint32_t"]),
    inl("string_insert", "void", &["$vec*", "size_t", "uint32_t", "$loc"]),
    inl("string_remove", "uint32_t", &["$vec*", "size_t", "$loc"]),
    inl("string_truncate", "void", &["$vec*", "size_t", "$loc"]),
    f("input", "$vec", &["$str", "$loc"]),
    // -- memory ([UNS-5], [HEAP-*])
    f("alloc", "void*", &["size_t", "size_t"]),
    f("obj_new_copy", "$obj_header*", &["const $type_info*", "size_t", "size_t", "const void*"]),
    f("weak_upgrade", "$obj_header*", &["$obj_header*"]),
    // -- the standard library's runtime functions (`Builtin::Runtime`)
    f("entropy", "uint64_t", &[]),
    RtFn { name: "process_exit", ret: "void", params: &["int64_t"], noreturn: true, inline: false },
    RtFn { name: "process_abort", ret: "void", params: &[], noreturn: true, inline: false },
    f("process_args", "$vec", &[]),
    f("process_args_os", "$vec", &[]),
    f("env_var", "bool", &["$str", "$vec*"]),
    f("fs_read", "int64_t", &["$str", "$vec*"]),
    f("fs_write", "int64_t", &["$str", "$span", "bool"]),
    f("fs_exists", "bool", &["$str"]),
    f("fs_metadata", "int64_t", &["$str", "int64_t*", "int64_t*"]),
    f("fs_remove", "int64_t", &["$str"]),
    f("fs_create_dir_all", "int64_t", &["$str"]),
    f("fs_read_dir", "int64_t", &["$str", "$vec*"]),
    f("file_open", "int64_t", &["$str", "int64_t", "int64_t*"]),
    f("file_read", "int64_t", &["int64_t", "$mutspan", "int64_t*"]),
    f("file_write", "int64_t", &["int64_t", "$span"]),
    f("file_flush", "int64_t", &["int64_t"]),
    f("file_close", "int64_t", &["int64_t"]),
    f("stdin_read_line", "int64_t", &["$vec*", "bool*"]),
    f("io_error_kind", "int64_t", &["int64_t"]),
    f("io_error_message", "void", &["int64_t", "$vec*"]),
    f("time_now", "int64_t", &[]),
    f("time_sleep", "void", &["int64_t"]),
    f("char_class", "bool", &["uint32_t", "int64_t"]),
    f("try_alloc", "void*", &["size_t", "size_t"]),
    f("try_realloc", "void*", &["void*", "size_t", "size_t", "size_t"]),
    f("system_alloc", "void*", &["size_t", "size_t"]),
    f("system_free", "void", &["void*", "size_t", "size_t"]),
    f("system_realloc", "void*", &["void*", "size_t", "size_t", "size_t"]),
    // -- text ([TXT-*])
    inl("str_eq", "bool", &["$str", "$str"]),
    f("str_cmp", "int", &["$str", "$str"]),
    inl("str_starts_with", "bool", &["$str", "$str"]),
    inl("str_ends_with", "bool", &["$str", "$str"]),
    inl("str_find", "int64_t", &["$str", "$str"]),
    inl("str_search", "size_t", &["$str", "$str", "size_t"]),
    inl("str_contains", "bool", &["$str", "$str"]),
    inl("str_is_char_boundary", "bool", &["$str", "size_t"]),
    f("str_rfind", "int64_t", &["$str", "$str"]),
    f("str_count", "int64_t", &["$str", "$str"]),
    f("str_replace", "$vec", &["$str", "$str", "$str"]),
    f("str_repeat", "$vec", &["$str", "int64_t"]),
    f("str_trim_start", "size_t", &["$str"]),
    f("str_trim_end", "size_t", &["$str"]),
    f("str_slice_ok", "bool", &["$str", "int64_t", "int64_t", "bool"]),
    f("str_to_upper", "$vec", &["$str"]),
    f("str_to_lower", "$vec", &["$str"]),
    f("str_char_count", "size_t", &["$str"]),
    f("str_char_at", "uint32_t", &["$str", "size_t"]),
    f("str_contains_char", "bool", &["$str", "uint32_t"]),
    inl("str_char_next", "uint32_t", &["$str", "int64_t*"]),
    inl("str_char_next_usize", "uint32_t", &["$str", "size_t*"]),
    inl("str_char_next_wide_usize", "uint32_t", &["const unsigned char*", "size_t*", "uint32_t"]),
    f("utf8_valid", "bool", &["$span"]),
    f("process_key", "uint64_t", &["int64_t"]),
    f("parse_signed_status", "uint8_t", &["$str", "int64_t"]),
    f("parse_signed_value", "int64_t", &["$str"]),
    f("parse_unsigned_status", "uint8_t", &["$str", "uint64_t"]),
    f("parse_unsigned_value", "uint64_t", &["$str"]),
    f("parse_i128_status", "uint8_t", &["$str"]),
    f("parse_i128_value", "$i128", &["$str"]),
    f("parse_u128_status", "uint8_t", &["$str"]),
    f("parse_u128_value", "$u128", &["$str"]),
    f("parse_i256_status", "uint8_t", &["$str"]),
    f("parse_i256_value", "$i256", &["$str"]),
    f("parse_u256_status", "uint8_t", &["$str"]),
    f("parse_u256_value", "$u256", &["$str"]),
    f("parse_float_status", "uint8_t", &["$str"]),
    f("parse_f64_value", "double", &["$str"]),
    f("parse_f32_value", "float", &["$str"]),
    f("parse_f16_value", "uint16_t", &["$str"]),
    f("parse_bool_status", "uint8_t", &["$str"]),
    f("parse_char_status", "uint8_t", &["$str"]),
    // -- numbers ([TYP-37], [STD-20], [STD-26], [RNG-3a])
    f("total_lt_f64", "bool", &["double", "double"]),
    f("total_lt_f32", "bool", &["float", "float"]),
    f("total_lt_f16", "bool", &["uint16_t", "uint16_t"]),
    f("range_contains_f64", "bool", &["double", "double", "double", "bool"]),
    f("range_contains_f32", "bool", &["float", "float", "float", "bool"]),
    f("range_clamp_f64", "double", &["double", "double", "double"]),
    f("range_clamp_f32", "float", &["float", "float", "float"]),
    f("range_count_i64", "size_t", &["int64_t", "int64_t", "int64_t"]),
    f("range_count_u64", "size_t", &["uint64_t", "uint64_t", "uint64_t"]),
    inl("range_nth_i64", "int64_t", &["int64_t", "int64_t", "size_t"]),
    inl("range_nth_u64", "uint64_t", &["uint64_t", "uint64_t", "size_t"]),
    f("range_count_i128", "size_t", &["$i128", "$i128", "$i128"]),
    f("range_count_u128", "size_t", &["$u128", "$u128", "$u128"]),
    f("range_nth_i128", "$i128", &["$i128", "$i128", "size_t"]),
    f("range_nth_u128", "$u128", &["$u128", "$u128", "size_t"]),
    // D-272 — the 128-bit operations that would be compiler-rt calls, which an MSVC link lacks.
    inl("ck_mul_i128", "bool", &["$i128", "$i128", "$i128*"]),
    inl("ck_mul_u128", "bool", &["$u128", "$u128", "$u128*"]),
    inl("ck_div_i128", "bool", &["$i128", "$i128", "$i128*"]),
    inl("ck_rem_i128", "bool", &["$i128", "$i128", "$i128*"]),
    inl("ck_floordiv_i128", "bool", &["$i128", "$i128", "$i128*"]),
    inl("ck_floorrem_i128", "bool", &["$i128", "$i128", "$i128*"]),
    inl("i128_div", "$i128", &["$i128", "$i128"]),
    inl("i128_rem", "$i128", &["$i128", "$i128"]),
    inl("i128_floordiv", "$i128", &["$i128", "$i128"]),
    inl("i128_floorrem", "$i128", &["$i128", "$i128"]),
    inl("u128_div", "$u128", &["$u128", "$u128"]),
    inl("u128_rem", "$u128", &["$u128", "$u128"]),
    inl("i128_to_f64", "double", &["$i128"]),
    inl("i128_to_f32", "float", &["$i128"]),
    inl("u128_to_f64", "double", &["$u128"]),
    inl("u128_to_f32", "float", &["$u128"]),
    inl("ftoi_i128", "$i128", &["double"]),
    inl("ftoi_u128", "$u128", &["double"]),
    inl("f16_to_f64", "double", &["uint16_t"]),
    f("f64_to_f16", "uint16_t", &["double"]),
    // -- formatting ([TYP-39], [LEX-19])
    f("fmt_i64", "void", &["$vec*", "int64_t"]),
    f("fmt_u64", "void", &["$vec*", "uint64_t"]),
    f("fmt_i128", "void", &["$vec*", "$i128"]),
    f("fmt_u128", "void", &["$vec*", "$u128"]),
    f("fmt_i256", "void", &["$vec*", "$i256"]),
    f("fmt_u256", "void", &["$vec*", "$u256"]),
    f("fmt_f64", "void", &["$vec*", "double"]),
    f("fmt_f32", "void", &["$vec*", "float"]),
    f("fmt_f16", "void", &["$vec*", "uint16_t"]),
    f("fmt_bool", "void", &["$vec*", "bool"]),
    f("fmt_char", "void", &["$vec*", "uint32_t"]),
    f("fmt_str", "void", &["$vec*", "$str"]),
    f("fmt_repr_str", "void", &["$vec*", "$str"]),
    f("fmt_repr_char", "void", &["$vec*", "uint32_t"]),
    f("fmt_handle", "void", &["$vec*", "const void*"]),
    f("fmt_spec_i64", "void", &["$vec*", "int64_t", "$fmt_spec"]),
    f("fmt_spec_u64", "void", &["$vec*", "uint64_t", "$fmt_spec"]),
    f("fmt_spec_i128", "void", &["$vec*", "$i128", "$fmt_spec"]),
    f("fmt_spec_u128", "void", &["$vec*", "$u128", "$fmt_spec"]),
    f("fmt_spec_i256", "void", &["$vec*", "$i256", "$fmt_spec"]),
    f("fmt_spec_u256", "void", &["$vec*", "$u256", "$fmt_spec"]),
    f("fmt_spec_f64", "void", &["$vec*", "double", "$fmt_spec"]),
    f("fmt_spec_f32", "void", &["$vec*", "float", "$fmt_spec"]),
    f("fmt_spec_f16", "void", &["$vec*", "uint16_t", "$fmt_spec"]),
    f("fmt_spec_str", "void", &["$vec*", "$str", "$fmt_spec"]),
    f("fmt_spec_bool", "void", &["$vec*", "bool", "$fmt_spec"]),
    f("fmt_spec_char", "void", &["$vec*", "uint32_t", "$fmt_spec"]),
    f("fmt_sink_open", "$vec", &["void*", "size_t"]),
    f("fmt_sink_close", "size_t", &["const $vec*"]),
    f("int_text", "$str", &["int64_t"]),
    f("uint_text", "$str", &["uint64_t"]),
    // -- exclusivity ([EXC-1], [EXC-15], [EXC-19])
    f("object_begin_write", "void", &["$obj_header*", "$loc"]),
    f("object_end_write", "void", &["$obj_header*", "$loc"]),
    f("object_begin_read", "void", &["$obj_header*", "$loc"]),
    f("object_end_read", "void", &["$obj_header*", "$loc"]),
    f("object_check_write", "void", &["$obj_header*", "$loc"]),
    f("object_check_read", "void", &["$obj_header*", "$loc"]),
    panic_fn("object_check_failed", &["$obj_header*", "bool", "$loc"]),
    f("access_begin_read", "void", &["$obj_header*", "const char*", "$loc"]),
    f("access_begin_write", "void", &["$obj_header*", "const char*", "$loc"]),
    f("access_end_read", "void", &["$obj_header*", "const char*", "$loc"]),
    f("access_end_write", "void", &["$obj_header*", "const char*", "$loc"]),
    panic_fn("panic_exclusivity", &["const char*", "$loc"]),
    panic_fn("panic_field_exclusivity", &["bool", "uint32_t", "const char*", "$loc"]),
    panic_fn("panic_field_exclusivity_debug", &["bool", "uint32_t", "const uint32_t*", "const char*", "$loc"]),
    f("debug_field_begin", "void", &["const uint32_t*", "bool", "$loc"]),
    f("debug_field_end", "void", &["const uint32_t*", "bool", "$loc"]),
    // -- the lifecycle ([RT-2], [FN-8], [FFI-22])
    f("rt_thread_attach", "void", &[]),
    f("rt_check_main_thread", "void", &[]),
    f("rt_static_begin", "int", &["uint32_t*", "const char*"]),
    f("rt_static_end", "void", &["uint32_t*"]),
    // -- arenas ([ARN-1]..[ARN-6], [JOB-5])
    f("arena_new", "void*", &["size_t"]),
    f("arena_alloc_copy", "void*", &["void*", "size_t", "size_t", "const void*"]),
    f("arena_alloc_uninit", "void*", &["void*", "size_t", "size_t", "size_t"]),
    f("arena_alloc_zeroed", "void*", &["void*", "size_t", "size_t", "size_t"]),
    f("arena_reset", "void", &["void*"]),
    f("arena_free", "void", &["void*"]),
    f("arena_mark", "void*", &["void*"]),
    f("arena_rewind", "void", &["void*", "void*"]),
    f("job_local_arena", "void*", &["$loc"]),
    f("fixed_arena_alloc_copy", "void*", &["void*", "size_t", "size_t*", "size_t", "size_t", "const void*"]),
    // -- locks ([THR-3], [THR-4]): inline in one runtime build, called in another
    inl("mutex_lock", "void", &["void*"]),
    inl("mutex_try_lock", "bool", &["void*"]),
    inl("mutex_unlock", "void", &["void*"]),
    inl("rwlock_acquire", "void", &["void*", "bool"]),
    inl("rwlock_try_acquire", "bool", &["void*", "bool"]),
    inl("rwlock_release", "void", &["void*", "bool"]),
    inl("lock_order_taken", "void", &["const void*", "$loc"]),
    inl("lock_order_freed", "void", &["const void*"]),
    // -- atomics ([THR-14], [THR-9])
    panic_fn("atomic_order_refused", &["uint8_t", "const char*", "$loc"]),
    f("atomic_wait32", "void", &["uint32_t*", "uint32_t"]),
    f("atomic_wake_one", "void", &["uint32_t*"]),
    f("atomic_wake_all", "void", &["uint32_t*"]),
    inl("atomic128_cas", "bool", &["volatile void*", "void*", "$u128"]),
    inl("atomic128_load", "$u128", &["volatile void*"]),
    inl("atomic128_swap", "$u128", &["volatile void*", "$u128"]),
    inl("atomic128_store", "void", &["volatile void*", "$u128"]),
    // -- threads, scopes, jobs and parallel loops ([THR-5], [THR-10], [THR-11], [JOB-1]..[JOB-3], [PAR-1])
    f("thread_start", "void", &["$thread*", "void (*)($thread*)", "void (*)($thread*)", "uint32_t", "uint32_t", "$loc"]),
    f("thread_join", "void", &["$thread*"]),
    f("thread_free", "void", &["$thread*"]),
    f("thread_detach", "void", &["$thread*"]),
    f("scope_new", "void*", &[]),
    f("scope_add", "void", &["void*", "$thread*"]),
    f("scope_join", "void", &["$thread*"]),
    f("scope_end", "void", &["void*"]),
    f(
        "job_submit",
        "uint64_t",
        &["uint32_t*", "void (*)(void*)", "void (*)(void*)", "const void*", "size_t", "size_t", "const void*", "size_t", "size_t", "size_t", "$loc"],
    ),
    f("job_wait", "void", &["uint64_t"]),
    f("job_scope_end", "void", &["uint32_t*"]),
    f(
        "parallel_for",
        "void",
        &[
            "int64_t",
            "int64_t",
            "int64_t",
            "void (*)(void*, int64_t, int64_t, void*)",
            "void (*)(void*, void*, const void*)",
            "void*",
            "void*",
            "size_t",
            "unsigned char*",
            "int64_t*",
            "uint32_t",
            "$loc",
        ],
    ),
    // -- pinned objects ([FFI-23], [HR-20]) and C++ objects ([FFI-39])
    f("obj_pin", "void", &["$obj_header*"]),
    f("obj_pin_relocatable", "uint64_t", &["$obj_header*", "void (*)(void)"]),
    f("obj_unpin_relocatable", "void", &["uint64_t"]),
    f("cpp_object_adopt", "$obj_header*", &["$obj_header*", "$loc"]),
    // -- foreign objects and threads ([FFI-36a], [FFI-33b])
    f("ffi_thread", "uint32_t", &[]),
    f("ffi_check_thread", "void", &["uint32_t", "const char*"]),
    f("ffi_adopt", "void*", &["void*", "$loc"]),
    f("ffi_release", "void", &["void*", "uint32_t"]),
    f("rt_set_args", "void", &["int", "char**"]),
    f("rt_init_v3", "int", &["const $rt_config*", "uint32_t"]),
    f("rt_shutdown", "void", &[]),
    f("backtrace_names", "void", &["const $frame_name*", "size_t"]),
    f("backtrace_sites", "void", &["const $call_site*", "size_t"]),
    f("stack_names", "void", &["const $code_name*", "size_t"]),
];

pub fn find(name: &str) -> &'static RtFn {
    FNS.iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("the LLVM backend's runtime table has no `{name}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The header's declaration of `name`, its parameters' types without their names.
    fn declared(header: &str, name: &str) -> Option<(String, Vec<String>)> {
        let symbol = ember_branding::runtime(name);
        // Without comments, and without preprocessor lines.
        let mut flat = String::new();
        let mut rest = header;
        while let Some(start) = rest.find("/*") {
            flat.push_str(&rest[..start]);
            rest = rest[start..].find("*/").map_or("", |end| &rest[start + end + 2..]);
        }
        flat.push_str(rest);
        let flat: String = flat
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .map(|line| line.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join(" ");
        for statement in flat.split(';') {
            let Some(at) = statement.find(&format!("{symbol}(")) else { continue };
            // What follows the end of a definition, or of a macro's use, before it.
            let before = statement[..at].rsplit(['{', '}', ')']).next().unwrap_or_default().trim();
            // A declaration: a type before the name, nothing else.
            if before.is_empty() || !before.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ' ' || c == '*') {
                continue;
            }
            let ret = before
                .split_whitespace()
                .filter(|word| !word.starts_with("EMBER_") && *word != "static" && *word != "inline")
                .collect::<Vec<_>>()
                .join(" ");
            let after = &statement[at + symbol.len() + 1..];
            // The parenthesis that closes the parameter list.
            let mut depth = 0;
            let close = after.char_indices().find_map(|(index, c)| match c {
                '(' => {
                    depth += 1;
                    None
                }
                ')' if depth == 0 => Some(index),
                ')' => {
                    depth -= 1;
                    None
                }
                _ => None,
            });
            let Some(close) = close else { continue };
            // The parameters: split at the commas outside a function pointer's own list.
            let mut depth = 0;
            let params = after[..close]
                .split(|c: char| {
                    match c {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    c == ',' && depth == 0
                })
                .map(str::trim)
                .filter(|param| !param.is_empty() && *param != "void")
                .map(|param| {
                    // A function pointer's name is inside its declarator: `void (*callback)(void)`.
                    if let Some(open) = param.find("(*") {
                        let close = param[open..].find(')').map_or(param.len(), |at| open + at);
                        return format!("{}(*{}", &param[..open], &param[close..]).replace("  ", " ");
                    }
                    // Drop the parameter's name: the last word, unless the type ends with `*`.
                    let param = param.replace(" *", "* ");
                    match param.rfind(|c: char| c == ' ' || c == '*') {
                        Some(split) if !param.ends_with('*') => param[..=split].trim().to_string(),
                        _ => param.trim().to_string(),
                    }
                })
                .collect();
            return Some((ret, params));
        }
        None
    }

    #[test]
    fn rt_5_every_prototype_is_the_headers() {
        let header_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime")
            .join(format!("{}_rt", ember_branding::SYMBOL_PREFIX))
            .join("include")
            .join(ember_branding::runtime_header());
        let header = std::fs::read_to_string(&header_path).expect("the runtime header");
        let spell = |text: &str| text.replace('$', ember_branding::RUNTIME_PREFIX);
        for entry in FNS {
            let Some((ret, params)) = declared(&header, entry.name) else {
                // A function a header macro instantiates (`EMBER_TEXT_CHAR_NEXT(ember_str_char_next,
                // int64_t)`): its instantiation names it.
                let instantiation = format!("({},", ember_branding::runtime(entry.name));
                assert!(entry.inline && header.contains(&instantiation), "`{}` is not declared in the runtime header", entry.name);
                continue;
            };
            assert_eq!(ret, spell(entry.ret), "the result of `{}`", entry.name);
            let ours: Vec<String> = entry.params.iter().map(|param| spell(param)).collect();
            assert_eq!(params, ours, "the parameters of `{}`", entry.name);
            // Every type the table names parses.
            let _ = c_type(entry.ret);
            for param in entry.params {
                let _ = c_type(param);
            }
        }
        assert!(header.contains("#define EMBER_RUNTIME_ABI 3"), "the runtime ABI the entry point names (`rt_init_v3`)");
    }
}
