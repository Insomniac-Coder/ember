/* The thunks of `import cpp "nlohmann_json.hpp"` ([FFI-3], [FFI-29]): the layout checks and the `extern "C"` functions Ember calls. */
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <memory>
#include <new>
#include <stdexcept>
#include <string>
#include <typeinfo>
#include <utility>
#if __has_include(<cxxabi.h>)
#include <cxxabi.h>
#endif
#define _Static_assert static_assert
#define _Alignof alignof

namespace {
thread_local std::string caught_type, caught_what;
[[maybe_unused]] std::string type_name(const char* name) {
#if __has_include(<cxxabi.h>)
    int status = 0;
    char* demangled = abi::__cxa_demangle(name, nullptr, nullptr, &status);
    if (status == 0 && demangled) { std::string out(demangled); std::free(demangled); return out; }
#endif
    std::string out(name);
    for (const char* tag : {"class ", "struct "}) { if (out.rfind(tag, 0) == 0) out.erase(0, std::strlen(tag)); }
    return out;
}
[[maybe_unused]] void caught() noexcept {
    try { throw; }
#if (defined(_MSC_VER) && defined(_CPPRTTI)) || (!defined(_MSC_VER) && (defined(__cpp_rtti) || defined(__GXX_RTTI)))
    catch (const std::exception& error) { caught_type = type_name(typeid(error).name()); caught_what = error.what(); }
#else
#define ember_CAUGHT(T) catch (const T& error) { caught_type = #T; caught_what = error.what(); }
    ember_CAUGHT(std::bad_array_new_length) ember_CAUGHT(std::bad_alloc)
    ember_CAUGHT(std::invalid_argument) ember_CAUGHT(std::domain_error) ember_CAUGHT(std::length_error) ember_CAUGHT(std::out_of_range) ember_CAUGHT(std::logic_error)
    ember_CAUGHT(std::range_error) ember_CAUGHT(std::overflow_error) ember_CAUGHT(std::underflow_error) ember_CAUGHT(std::runtime_error)
    ember_CAUGHT(std::exception)
#undef ember_CAUGHT
#endif
    catch (...) { caught_type = "an exception that is no std::exception"; caught_what.clear(); }
}
template <class T> [[maybe_unused]] T zero() noexcept { union { unsigned char bytes[sizeof(T)]; T value; } none = {}; return none.value; }
[[noreturn]] [[maybe_unused]] void crossed(const char* where) noexcept {
    caught();
    std::fprintf(stderr, "a C++ exception would cross into Ember from %s: %s: %s [FFI-39e]\n", where, caught_type.c_str(), caught_what.c_str());
    std::fflush(stderr);
    std::abort();
}
}

extern "C" const char* ember_cpp_bind_error_type(void) noexcept { return caught_type.c_str(); }
extern "C" const char* ember_cpp_bind_error_what(void) noexcept { return caught_what.c_str(); }
#include "nlohmann_json.hpp"

_Static_assert(sizeof(nlohmann::detail::position_t) == 24, "position_t: size");
_Static_assert(_Alignof(nlohmann::detail::position_t) == 8, "position_t: alignment");
_Static_assert(offsetof(nlohmann::detail::position_t, chars_read_total) == 0, "position_t.chars_read_total: offset");
_Static_assert(offsetof(nlohmann::detail::position_t, chars_read_current_line) == 8, "position_t.chars_read_current_line: offset");
_Static_assert(offsetof(nlohmann::detail::position_t, lines_read) == 16, "position_t.lines_read: offset");
extern "C" void ember_cpp_bind_0(nlohmann::detail2::begin_tag* self) noexcept { delete self; }
extern "C" void ember_cpp_bind_1(nlohmann::detail2::end_tag* self) noexcept { delete self; }
extern "C" unsigned long ember_cpp_bind_2(int* failed) noexcept { try { *failed = 0; return ::nlohmann::detail::concat_length(); } catch (...) { *failed = 1; caught(); return zero<unsigned long>(); } }
extern "C" void ember_cpp_bind_3(nlohmann::detail::exception* self) noexcept { delete self; }
extern "C" const char * ember_cpp_bind_4(const nlohmann::detail::exception* self) noexcept { return self->what(); }
extern "C" void ember_cpp_bind_5(nlohmann::detail::parse_error* self) noexcept { delete self; }
extern "C" const char * ember_cpp_bind_6(const nlohmann::detail::parse_error* self) noexcept { return self->what(); }
extern "C" void ember_cpp_bind_7(nlohmann::detail::invalid_iterator* self) noexcept { delete self; }
extern "C" const char * ember_cpp_bind_8(const nlohmann::detail::invalid_iterator* self) noexcept { return self->what(); }
extern "C" void ember_cpp_bind_9(nlohmann::detail::type_error* self) noexcept { delete self; }
extern "C" const char * ember_cpp_bind_10(const nlohmann::detail::type_error* self) noexcept { return self->what(); }
extern "C" void ember_cpp_bind_11(nlohmann::detail::out_of_range* self) noexcept { delete self; }
extern "C" const char * ember_cpp_bind_12(const nlohmann::detail::out_of_range* self) noexcept { return self->what(); }
extern "C" void ember_cpp_bind_13(nlohmann::detail::other_error* self) noexcept { delete self; }
extern "C" const char * ember_cpp_bind_14(const nlohmann::detail::other_error* self) noexcept { return self->what(); }
extern "C" void ember_cpp_bind_15(nlohmann::detail::from_json_fn* self) noexcept { delete self; }
extern "C" void ember_cpp_bind_16(nlohmann::detail::to_json_fn* self) noexcept { delete self; }
extern "C" unsigned long ember_cpp_bind_17(unsigned long a0, unsigned long a1) noexcept { return ::nlohmann::detail::combine(a0, a1); }
extern "C" void ember_cpp_bind_18(nlohmann::detail::input_stream_adapter* self) noexcept { delete self; }
extern "C" int ember_cpp_bind_19(nlohmann::detail::input_stream_adapter* self, int* failed) noexcept { try { *failed = 0; return self->get_character(); } catch (...) { *failed = 1; caught(); return zero<int>(); } }
extern "C" bool ember_cpp_bind_20(int a0) noexcept { return ::nlohmann::detail::little_endianness(a0); }
_Static_assert(sizeof(nlohmann::detail::primitive_iterator_t) == 8, "primitive_iterator_t: size");
_Static_assert(_Alignof(nlohmann::detail::primitive_iterator_t) == 8, "primitive_iterator_t: alignment");
extern "C" long ember_cpp_bind_21(const nlohmann::detail::primitive_iterator_t* self) noexcept { return self->get_value(); }
extern "C" void ember_cpp_bind_22(nlohmann::detail::primitive_iterator_t* self) noexcept { self->set_begin(); }
extern "C" void ember_cpp_bind_23(nlohmann::detail::primitive_iterator_t* self) noexcept { self->set_end(); }
extern "C" bool ember_cpp_bind_24(const nlohmann::detail::primitive_iterator_t* self) noexcept { return self->is_begin(); }
extern "C" bool ember_cpp_bind_25(const nlohmann::detail::primitive_iterator_t* self) noexcept { return self->is_end(); }
extern "C" void ember_cpp_bind_26(nlohmann::detail::primitive_iterator_t* self, long a0, nlohmann::detail::primitive_iterator_t* out) noexcept { ::new (static_cast<void*>(out)) nlohmann::detail::primitive_iterator_t(self->operator+(a0)); }
extern "C" void ember_cpp_bind_27(nlohmann::detail::primitive_iterator_t* self, long a0) noexcept { self->operator+=(a0); }
extern "C" void ember_cpp_bind_28(nlohmann::detail::primitive_iterator_t* self, long a0) noexcept { self->operator-=(a0); }
extern "C" bool ember_cpp_bind_29(const nlohmann::detail::primitive_iterator_t* self, nlohmann::detail::primitive_iterator_t a1) noexcept { return (*self) == (a1); }
extern "C" long ember_cpp_bind_30(const nlohmann::detail::primitive_iterator_t* self, nlohmann::detail::primitive_iterator_t a1) noexcept { return (*self) - (a1); }
extern "C" void ember_cpp_bind_31(nlohmann::detail::json_default_base* self) noexcept { delete self; }
_Static_assert(sizeof(nlohmann::detail::dtoa_impl::diyfp) == 16, "diyfp: size");
_Static_assert(_Alignof(nlohmann::detail::dtoa_impl::diyfp) == 8, "diyfp: alignment");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::diyfp, f) == 0, "diyfp.f: offset");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::diyfp, e) == 8, "diyfp.e: offset");
extern "C" void ember_cpp_bind_32(unsigned long a0, int a1, nlohmann::detail::dtoa_impl::diyfp* out) noexcept { ::new (static_cast<void*>(out)) nlohmann::detail::dtoa_impl::diyfp(a0, a1); }
extern "C" void ember_cpp_bind_33(const nlohmann::detail::dtoa_impl::diyfp a0, const nlohmann::detail::dtoa_impl::diyfp a1, nlohmann::detail::dtoa_impl::diyfp* out) noexcept { ::new (static_cast<void*>(out)) nlohmann::detail::dtoa_impl::diyfp(::nlohmann::detail::dtoa_impl::diyfp::sub(a0, a1)); }
extern "C" void ember_cpp_bind_34(const nlohmann::detail::dtoa_impl::diyfp a0, const nlohmann::detail::dtoa_impl::diyfp a1, nlohmann::detail::dtoa_impl::diyfp* out) noexcept { ::new (static_cast<void*>(out)) nlohmann::detail::dtoa_impl::diyfp(::nlohmann::detail::dtoa_impl::diyfp::mul(a0, a1)); }
extern "C" void ember_cpp_bind_35(nlohmann::detail::dtoa_impl::diyfp a0, nlohmann::detail::dtoa_impl::diyfp* out) noexcept { ::new (static_cast<void*>(out)) nlohmann::detail::dtoa_impl::diyfp(::nlohmann::detail::dtoa_impl::diyfp::normalize(a0)); }
extern "C" void ember_cpp_bind_36(const nlohmann::detail::dtoa_impl::diyfp a0, const int a1, nlohmann::detail::dtoa_impl::diyfp* out) noexcept { ::new (static_cast<void*>(out)) nlohmann::detail::dtoa_impl::diyfp(::nlohmann::detail::dtoa_impl::diyfp::normalize_to(a0, a1)); }
_Static_assert(sizeof(nlohmann::detail::dtoa_impl::boundaries) == 48, "boundaries: size");
_Static_assert(_Alignof(nlohmann::detail::dtoa_impl::boundaries) == 8, "boundaries: alignment");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::boundaries, w) == 0, "boundaries.w: offset");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::boundaries, minus) == 16, "boundaries.minus: offset");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::boundaries, plus) == 32, "boundaries.plus: offset");
_Static_assert(sizeof(nlohmann::detail::dtoa_impl::cached_power) == 16, "cached_power: size");
_Static_assert(_Alignof(nlohmann::detail::dtoa_impl::cached_power) == 8, "cached_power: alignment");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::cached_power, f) == 0, "cached_power.f: offset");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::cached_power, e) == 8, "cached_power.e: offset");
_Static_assert(offsetof(nlohmann::detail::dtoa_impl::cached_power, k) == 12, "cached_power.k: offset");
extern "C" void ember_cpp_bind_37(int a0, nlohmann::detail::dtoa_impl::cached_power* out, int* failed) noexcept { try { *failed = 0; ::new (static_cast<void*>(out)) nlohmann::detail::dtoa_impl::cached_power(::nlohmann::detail::dtoa_impl::get_cached_power_for_binary_exponent(a0)); } catch (...) { *failed = 1; caught(); std::memset(static_cast<void*>(out), 0, sizeof *out); } }
extern "C" int ember_cpp_bind_38(const unsigned int a0, unsigned int* a1, int* failed) noexcept { try { *failed = 0; return ::nlohmann::detail::dtoa_impl::find_largest_pow10(a0, *a1); } catch (...) { *failed = 1; caught(); return zero<int>(); } }
extern "C" void ember_cpp_bind_39(char *a0, int a1, unsigned long a2, unsigned long a3, unsigned long a4, unsigned long a5, int* failed) noexcept { try { *failed = 0; ::nlohmann::detail::dtoa_impl::grisu2_round(a0, a1, a2, a3, a4, a5); } catch (...) { *failed = 1; caught(); } }
extern "C" void ember_cpp_bind_40(char *a0, int* a1, int* a2, nlohmann::detail::dtoa_impl::diyfp a3, nlohmann::detail::dtoa_impl::diyfp a4, nlohmann::detail::dtoa_impl::diyfp a5, int* failed) noexcept { try { *failed = 0; ::nlohmann::detail::dtoa_impl::grisu2_digit_gen(a0, *a1, *a2, a3, a4, a5); } catch (...) { *failed = 1; caught(); } }
extern "C" void ember_cpp_bind_41(char *a0, int* a1, int* a2, nlohmann::detail::dtoa_impl::diyfp a3, nlohmann::detail::dtoa_impl::diyfp a4, nlohmann::detail::dtoa_impl::diyfp a5, int* failed) noexcept { try { *failed = 0; ::nlohmann::detail::dtoa_impl::grisu2(a0, *a1, *a2, a3, a4, a5); } catch (...) { *failed = 1; caught(); } }
extern "C" char * ember_cpp_bind_42(char *a0, int a1, int* failed) noexcept { try { *failed = 0; return ::nlohmann::detail::dtoa_impl::append_exponent(a0, a1); } catch (...) { *failed = 1; caught(); return zero<char *>(); } }
extern "C" char * ember_cpp_bind_43(char *a0, int a1, int a2, int a3, int a4, int* failed) noexcept { try { *failed = 0; return ::nlohmann::detail::dtoa_impl::format_buffer(a0, a1, a2, a3, a4); } catch (...) { *failed = 1; caught(); return zero<char *>(); } }
extern "C" void ember_cpp_bind_44(nlohmann::basic_json<>* self) noexcept { delete self; }
extern "C" nlohmann::basic_json<>* ember_cpp_bind_45(int* failed) noexcept { try { *failed = 0; return new nlohmann::basic_json<>(::nlohmann::basic_json<>::meta()); } catch (...) { *failed = 1; caught(); return zero<nlohmann::basic_json<>*>(); } }
extern "C" const char* ember_cpp_bind_bridge_0_data(void* s) noexcept { return static_cast<std::basic_string<char>*>(s)->data(); }
extern "C" std::size_t ember_cpp_bind_bridge_0_size(void* s) noexcept { return static_cast<std::basic_string<char>*>(s)->size(); }
extern "C" void ember_cpp_bind_bridge_0_destroy(void* s) noexcept { delete static_cast<std::basic_string<char>*>(s); }
extern "C" void* ember_cpp_bind_46(const nlohmann::basic_json<>* self, const int a0, const char a1, const bool a2, const nlohmann::detail::error_handler_t a3, int* failed) noexcept { try { *failed = 0; return static_cast<void*>(new std::basic_string<char>(self->dump(a0, a1, a2, a3))); } catch (...) { *failed = 1; caught(); return zero<void*>(); } }
extern "C" nlohmann::detail::value_t ember_cpp_bind_47(const nlohmann::basic_json<>* self) noexcept { return self->type(); }
extern "C" bool ember_cpp_bind_48(const nlohmann::basic_json<>* self) noexcept { return self->is_primitive(); }
extern "C" bool ember_cpp_bind_49(const nlohmann::basic_json<>* self) noexcept { return self->is_structured(); }
extern "C" bool ember_cpp_bind_50(const nlohmann::basic_json<>* self) noexcept { return self->is_null(); }
extern "C" bool ember_cpp_bind_51(const nlohmann::basic_json<>* self) noexcept { return self->is_boolean(); }
extern "C" bool ember_cpp_bind_52(const nlohmann::basic_json<>* self) noexcept { return self->is_number(); }
extern "C" bool ember_cpp_bind_53(const nlohmann::basic_json<>* self) noexcept { return self->is_number_integer(); }
extern "C" bool ember_cpp_bind_54(const nlohmann::basic_json<>* self) noexcept { return self->is_number_unsigned(); }
extern "C" bool ember_cpp_bind_55(const nlohmann::basic_json<>* self) noexcept { return self->is_number_float(); }
extern "C" bool ember_cpp_bind_56(const nlohmann::basic_json<>* self) noexcept { return self->is_object(); }
extern "C" bool ember_cpp_bind_57(const nlohmann::basic_json<>* self) noexcept { return self->is_array(); }
extern "C" bool ember_cpp_bind_58(const nlohmann::basic_json<>* self) noexcept { return self->is_string(); }
extern "C" bool ember_cpp_bind_59(const nlohmann::basic_json<>* self) noexcept { return self->is_binary(); }
extern "C" bool ember_cpp_bind_60(const nlohmann::basic_json<>* self) noexcept { return self->is_discarded(); }
extern "C" bool ember_cpp_bind_61(const nlohmann::basic_json<>* self) noexcept { return self->empty(); }
extern "C" unsigned long ember_cpp_bind_62(const nlohmann::basic_json<>* self) noexcept { return self->size(); }
extern "C" unsigned long ember_cpp_bind_63(const nlohmann::basic_json<>* self) noexcept { return self->max_size(); }
extern "C" void ember_cpp_bind_64(nlohmann::basic_json<>* self) noexcept { self->clear(); }
extern "C" const char * ember_cpp_bind_65(const nlohmann::basic_json<>* self) noexcept { return self->type_name(); }
extern "C" nlohmann::basic_json<>* ember_cpp_bind_66(const nlohmann::basic_json<>* self, int* failed) noexcept { try { *failed = 0; return new nlohmann::basic_json<>(self->flatten()); } catch (...) { *failed = 1; caught(); return zero<nlohmann::basic_json<>*>(); } }
extern "C" nlohmann::basic_json<>* ember_cpp_bind_67(const nlohmann::basic_json<>* self, int* failed) noexcept { try { *failed = 0; return new nlohmann::basic_json<>(self->unflatten()); } catch (...) { *failed = 1; caught(); return zero<nlohmann::basic_json<>*>(); } }
extern "C" void ember_cpp_bind_68(nlohmann::basic_json<>* self, const nlohmann::basic_json<>* a0, int* failed) noexcept { try { *failed = 0; self->patch_inplace(*a0); } catch (...) { *failed = 1; caught(); } }
extern "C" nlohmann::basic_json<>* ember_cpp_bind_69(const nlohmann::basic_json<>* self, const nlohmann::basic_json<>* a0, int* failed) noexcept { try { *failed = 0; return new nlohmann::basic_json<>(self->patch(*a0)); } catch (...) { *failed = 1; caught(); return zero<nlohmann::basic_json<>*>(); } }
extern "C" nlohmann::basic_json<>* ember_cpp_bind_70(const nlohmann::basic_json<>* a0, const nlohmann::basic_json<>* a1, void* a2, int* failed) noexcept { try { *failed = 0; return new nlohmann::basic_json<>(::nlohmann::basic_json<>::diff(*a0, *a1, *static_cast<const std::basic_string<char>*>(a2))); } catch (...) { *failed = 1; caught(); return zero<nlohmann::basic_json<>*>(); } }
extern "C" void ember_cpp_bind_71(nlohmann::basic_json<>* self, const nlohmann::basic_json<>* a0, int* failed) noexcept { try { *failed = 0; self->merge_patch(*a0); } catch (...) { *failed = 1; caught(); } }
