//! `[FFI-28]`, `[FFI-31]`, `[FFI-31a]`, `[FFI-31c]`, `[ABI-3]` — shared libraries: a package built
//! as `kind = "cdylib"`, loaded at run time by a C host through `ember_module_init` and
//! `ember_module_shutdown`, alone or beside another Ember library sharing one runtime.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use ember_build::{LinkRequest, Profile, Toolchain};

const EMBER: &str = env!("CARGO_BIN_EXE_ember");
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// `text` with the runtime's prefix where it says `ember_` (the C below names runtime symbols).
fn branded(text: &str) -> String {
    text.replace("@RT@", &ember_branding::runtime(""))
}

fn scratch() -> PathBuf {
    let path = std::env::temp_dir().join(format!("{}-cdylib-{}-{}",
        ember_branding::CLI_NAME, std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch directory");
    path
}

fn toolchain() -> Toolchain {
    let requested = std::env::var(ember_branding::cc_var()).ok();
    Toolchain::detect(requested.as_deref()).expect("a C toolchain")
}

/// A package in `dir`: its manifest and its `src/lib.em`.
fn package(dir: &Path, manifest: &str, source: &str) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join(ember_branding::MANIFEST), manifest).unwrap();
    std::fs::write(dir.join("src").join(ember_branding::source_file("lib")), source).unwrap();
}

fn ember(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(EMBER).args(args).current_dir(cwd).output().expect("ember starts")
}

fn assert_success(output: &std::process::Output, step: &str) {
    assert!(output.status.success(), "{step} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

/// The library file a package named `name` builds, in `lib`.
fn library_file(lib: &Path, name: &str) -> PathBuf {
    lib.join(if cfg!(windows) { format!("{name}.dll") } else { format!("lib{name}.so") })
}

/// Windows: no error box for a program this test starts (D-252), inherited by every child.
#[cfg(windows)]
#[allow(unsafe_code)]
fn quiet_errors() {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetErrorMode(mode: u32) -> u32;
    }
    // SAFETY: SetErrorMode only sets this process's error mode flags.
    unsafe {
        SetErrorMode(0x0001 | 0x0002 | 0x8000);
    }
}

#[cfg(not(windows))]
fn quiet_errors() {}

/// The C host's common part: loading a library and finding a symbol, on both systems.
const HOST_PRELUDE: &str = r#"#if !defined(_WIN32)
#define _DEFAULT_SOURCE /* usleep under -std=c11 */
#endif
#include "@RT@runtime/@RT@rt.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
typedef HMODULE library;
static library load(const char* path) { return LoadLibraryA(path); }
static void* find(library l, const char* name) { return (void*)GetProcAddress(l, name); }
static void unload(library l) { FreeLibrary(l); }
#else
#include <dlfcn.h>
#include <pthread.h>
#include <unistd.h>
typedef void* library;
static library load(const char* path) { return dlopen(path, RTLD_NOW | RTLD_LOCAL); }
static void* find(library l, const char* name) { return dlsym(l, name); }
static void unload(library l) { dlclose(l); }
#endif
typedef int (*module_init_fn)(const @RT@rt_config*);
typedef void (*module_shutdown_fn)(void);
typedef int32_t (*value_fn)(void);
typedef int32_t (*unary_fn)(int32_t);
static char g_log[8192];
static void host_log(int level, const char* msg, size_t len) {
    (void)level;
    size_t used = strlen(g_log);
    size_t room = sizeof g_log - used - 1;
    memcpy(g_log + used, msg, len < room ? len : room);
    g_log[used + (len < room ? len : room)] = '\0';
}
static void host_panic(const char* msg, size_t len) {
    printf("panic: %.*s\n", (int)len, msg);
    fflush(stdout);
    exit(0);
}
"#;

/// Compile `source` (a C host) into an executable in `dir` against the runtime header in `lib`.
fn link_host(toolchain: &Toolchain, dir: &Path, lib: &Path, name: &str, source: &str) -> PathBuf {
    let file = dir.join(format!("{name}.c"));
    std::fs::write(&file, branded(source)).unwrap();
    let exe = dir.join(if cfg!(windows) { format!("{name}.exe") } else { name.to_string() });
    let obj = dir.join(format!("{name}-obj"));
    std::fs::create_dir_all(&obj).unwrap();
    let libraries: Vec<String> = if cfg!(windows) { Vec::new() } else { vec!["dl".to_string()] };
    ember_build::compile_and_link_program(toolchain, &LinkRequest {
        sources: &[file], include_dirs: &[lib.to_path_buf()], output: exe.clone(), profile: Profile::Debug, obj_dir: obj,
    }, &[], &libraries).expect("the C host compiles");
    exe
}

fn run_host(exe: &Path, args: &[&str], cwd: &Path) -> std::process::Output {
    quiet_errors();
    ember_build::output_started(Command::new(exe).args(args).current_dir(cwd)).expect("the C host runs")
}

const PLUGIN: &str = r#"class Node:
    name: String
    next: Option[Node]

fn helper(x: i32) -> i32:
    return x * 2

@export("plugin_double")
fn double(x: i32) -> i32:
    return helper(x)

@export("plugin_on_main", threads=main)
fn on_main() -> i32:
    return 7

@export("plugin_leak")
fn leak() -> i32:
    a = Node("a", None)
    b = Node("b", Some(a))
    a.next = Some(b)
    return 1

@export("plugin-odd")
fn odd() -> i32:
    return 9
"#;

/// `[FFI-28]` — a `cdylib` exports its `@export` symbols, `ember_module_init` and
/// `ember_module_shutdown`, and nothing else: not its own functions, not its runtime's.
/// `[FFI-31]` — the host initialises the module, calls it from two threads, and shuts it down,
/// which runs the debug leak report (to the host's log) and detaches every thread; `[FFI-31a]`
/// a thread the module attached ends after the library is unloaded, which a TLS destructor in
/// the module's code would not survive; a second load starts afresh. `[FFI-33]`: a `main` export
/// is its creator's, and panics before `ember_module_init` and on another thread.
#[test]
fn a_cdylib_is_loaded_initialised_and_unloaded_by_a_c_host() {
    let dir = scratch();
    let toolchain = toolchain();
    package(&dir.join("plugin"), "[package]\nname = \"plugin\"\nkind = \"cdylib\"\n", PLUGIN);
    let built = ember(&["build"], &dir.join("plugin"));
    assert_success(&built, "the cdylib build");
    let lib = dir.join("plugin/target/debug/lib");
    let library = library_file(&lib, "plugin");
    assert!(library.is_file(), "no library at {}", library.display());
    let header = std::fs::read_to_string(lib.join("plugin.h")).unwrap();
    for declaration in ["int32_t plugin_double(int32_t", "int @RT@module_init(const @RT@rt_config* cfg);",
        "void @RT@module_shutdown(void);", "/* threads: main */"] {
        let declaration = branded(declaration);
        assert!(header.contains(&declaration), "the header lacks `{declaration}`:\n{header}");
    }

    let path = library.to_string_lossy().replace('\\', "\\\\");
    let host = format!(r#"{HOST_PRELUDE}
static library g_lib;
static volatile int g_stage;
#if defined(_WIN32)
static DWORD WINAPI worker(LPVOID unused) {{
#else
static void* worker(void* unused) {{
#endif
    (void)unused;
    if (g_stage == 3) {{
        ((value_fn)find(g_lib, "plugin_on_main"))(); /* a `main` export on another thread */
        return 0;
    }}
    unary_fn twice = (unary_fn)find(g_lib, "plugin_double");
    g_stage = twice(5) == 10 ? 1 : -1;
    while (g_stage != 2) {{
#if defined(_WIN32)
        Sleep(1);
#else
        usleep(1000);
#endif
    }}
    return 0;
}}

int main(int argc, char** argv) {{
    const char* mode = argc > 1 ? argv[1] : "flow";
    @RT@rt_config cfg = @RT@rt_config_default();
    cfg.log = host_log;
    cfg.on_panic = host_panic;
    g_lib = load("{path}");
    if (!g_lib) return 2;
    module_init_fn init = (module_init_fn)find(g_lib, "@RT@module_init");
    module_shutdown_fn shutdown = (module_shutdown_fn)find(g_lib, "@RT@module_shutdown");
    value_fn on_main = (value_fn)find(g_lib, "plugin_on_main");
    value_fn leak = (value_fn)find(g_lib, "plugin_leak");
    unary_fn twice = (unary_fn)find(g_lib, "plugin_double");
    value_fn odd = (value_fn)find(g_lib, "plugin-odd"); /* [MNG-2]: exactly the symbol written */
    if (!init || !shutdown || !on_main || !leak || !twice || !odd) return 3;
    /* Nothing but the exports and the entry points: no runtime symbol, no private function. */
    if (find(g_lib, "@RT@rt_init_v2") || find(g_lib, "@RT@alloc") || find(g_lib, "@RT@rt_shutdown")) return 4;
    if (strcmp(mode, "before") == 0) {{ on_main(); return 5; }}
    if (init(&cfg) != 0) return 6;
    if (strcmp(mode, "other") == 0) {{
        g_stage = 3;
#if defined(_WIN32)
        HANDLE t = CreateThread(NULL, 0, worker, NULL, 0, NULL);
        WaitForSingleObject(t, INFINITE);
#else
        pthread_t t;
        pthread_create(&t, NULL, worker, NULL);
        pthread_join(t, NULL);
#endif
        return 7;
    }}
    if (twice(21) != 42 || on_main() != 7 || odd() != 9) return 8;
    if (init(&cfg) != 0 || on_main() != 7) return 9; /* a second init changes nothing */
#if defined(_WIN32)
    HANDLE thread = CreateThread(NULL, 0, worker, NULL, 0, NULL);
#else
    pthread_t thread;
    pthread_create(&thread, NULL, worker, NULL);
#endif
    while (g_stage == 0) {{
#if defined(_WIN32)
        Sleep(1);
#else
        usleep(1000);
#endif
    }}
    if (g_stage != 1) return 10;
    if (leak() != 1) return 11;
    shutdown();
    if (strstr(g_log, "reference cycle detected") == NULL) return 12;
    unload(g_lib);
    g_stage = 2; /* the attached thread ends with the library gone */
#if defined(_WIN32)
    WaitForSingleObject(thread, INFINITE);
#else
    pthread_join(thread, NULL);
#endif
    g_lib = load("{path}");
    init = (module_init_fn)find(g_lib, "@RT@module_init");
    shutdown = (module_shutdown_fn)find(g_lib, "@RT@module_shutdown");
    on_main = (value_fn)find(g_lib, "plugin_on_main");
    if (init(&cfg) != 0 || on_main() != 7) return 13;
    shutdown();
    unload(g_lib);
    printf("ok\n");
    return 0;
}}
"#);
    let exe = link_host(&toolchain, &dir, &lib, "host", &host);
    let flow = run_host(&exe, &["flow"], &dir);
    let stdout = String::from_utf8_lossy(&flow.stdout);
    assert!(flow.status.success() && stdout.trim() == "ok", "the host's flow failed ({:?}): {stdout}{}",
        flow.status.code(), String::from_utf8_lossy(&flow.stderr));
    assert!(!String::from_utf8_lossy(&flow.stderr).contains("reference cycle"), "the leak report bypassed the host's log");
    // Before `ember_module_init` the host's `on_panic` is not known yet: the panic is on stderr.
    let before = run_host(&exe, &["before"], &dir);
    assert!(!before.status.success());
    assert!(String::from_utf8_lossy(&before.stderr).contains(&branded("export called before @RT@module_init")),
        "{}", String::from_utf8_lossy(&before.stderr));
    let other = run_host(&exe, &["other"], &dir);
    assert!(String::from_utf8_lossy(&other.stdout).contains("panic: export requires the module initialization thread"),
        "{}", String::from_utf8_lossy(&other.stdout));
    // `[FFI-31a]` — and the runtime the module carries registers no thread-exit callback at all.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).unwrap();
    let runtime_source = std::fs::read_to_string(root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX))
        .join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))).unwrap();
    for destructor in ["pthread_key_create", "FlsAlloc", "tss_create", "DLL_THREAD_DETACH"] {
        assert!(!runtime_source.contains(destructor), "the runtime uses `{destructor}`");
    }
    let mut exports = if cfg!(windows) {
        pe_exports(&std::fs::read(&library).unwrap())
    } else {
        let listed = Command::new("nm").args(["-D", "--defined-only"]).arg(&library).output().expect("nm runs");
        String::from_utf8_lossy(&listed.stdout).lines().filter_map(|line| line.split_whitespace().nth(2).map(str::to_string)).collect()
    };
    exports.sort();
    let expected = ["@RT@module_init", "@RT@module_shutdown", "plugin-odd", "plugin_double", "plugin_leak", "plugin_on_main"].map(branded);
    assert_eq!(exports, expected);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[FFI-31c]` — two Ember libraries built with `[build] runtime = "shared"` share the one runtime
/// of the process: it is loaded once, beside them, the host reaches it too, and it lives until
/// the last module's shutdown (an at-exit hook the host registers runs then, not before).
/// `[FFI-31b]` still holds across their exports: an owning value is `E5015`.
#[test]
fn shared_libraries_share_one_runtime() {
    let dir = scratch();
    let toolchain = toolchain();
    let manifest = |name: &str| format!("[package]\nname = \"{name}\"\nkind = \"cdylib\"\n[build]\nruntime = \"shared\"\n");
    package(&dir.join("first"), &manifest("first"),
        "@export(\"first_value\", threads=main)\nfn first_value() -> i32:\n    return 1\n");
    package(&dir.join("second"), &manifest("second"),
        "@export(\"second_value\", threads=creator)\nfn second_value() -> i32:\n    return 2\n");
    let out = dir.join("out");
    let out_arg = out.to_string_lossy().into_owned();
    for name in ["first", "second"] {
        let built = ember(&["build", "--out-dir", out_arg.as_str()], &dir.join(name));
        assert_success(&built, name);
    }
    let lib = out.join("debug/lib");
    let runtime = library_file(&lib, &format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    assert!(runtime.is_file(), "no shared runtime at {}", runtime.display());
    // The host and the shared runtime sit together, as a host's own directory would hold them.
    let first = library_file(&lib, "first").to_string_lossy().replace('\\', "\\\\");
    let second = library_file(&lib, "second").to_string_lossy().replace('\\', "\\\\");
    let runtime_name = runtime.file_name().unwrap().to_string_lossy().into_owned();
    let host = format!(r#"{HOST_PRELUDE}
static int g_hooked;
static void hook(void* data) {{ (void)data; g_hooked += 1; }}
typedef int (*at_exit_fn)(void (*)(void*), void*);
int main(void) {{
    @RT@rt_config cfg = @RT@rt_config_default();
    cfg.on_panic = host_panic;
    library a = load("{first}");
    library b = load("{second}");
    if (!a || !b) return 2;
#if defined(_WIN32)
    library rt = GetModuleHandleA("{runtime_name}");
#else
    library rt = dlopen("{runtime_name}", RTLD_NOW | RTLD_NOLOAD);
#endif
    if (!rt) return 3; /* one runtime, loaded with the modules */
    at_exit_fn at_exit = (at_exit_fn)find(rt, "@RT@rt_at_exit");
    module_init_fn init_a = (module_init_fn)find(a, "@RT@module_init");
    module_init_fn init_b = (module_init_fn)find(b, "@RT@module_init");
    module_shutdown_fn shutdown_a = (module_shutdown_fn)find(a, "@RT@module_shutdown");
    module_shutdown_fn shutdown_b = (module_shutdown_fn)find(b, "@RT@module_shutdown");
    value_fn first = (value_fn)find(a, "first_value");
    value_fn second = (value_fn)find(b, "second_value");
    if (!at_exit || !init_a || !init_b || !shutdown_a || !shutdown_b || !first || !second) return 4;
    if (init_a(&cfg) != 0 || init_b(&cfg) != 0) return 5;
    if (at_exit(hook, NULL) != 0) return 6;
    if (first() != 1 || second() != 2) return 7;
    shutdown_a();
    if (g_hooked != 0 || second() != 2) return 8; /* the runtime lives on for the second */
    shutdown_b();
    if (g_hooked != 1) return 9; /* the last module's shutdown ended it */
    unload(a);
    unload(b);
    printf("ok\n");
    return 0;
}}
"#);
    let exe = link_host(&toolchain, &lib, &lib, "host", &host);
    let output = run_host(&exe, &[], &lib);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success() && stdout.trim() == "ok", "the shared-runtime host failed ({:?}): {stdout}{}",
        output.status.code(), String::from_utf8_lossy(&output.stderr));

    package(&dir.join("owning"), &manifest("owning"),
        "@export(\"owning_text\")\nfn owning_text() -> String:\n    return String()\n");
    let refused = ember(&["build"], &dir.join("owning"));
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("error[E5015]"), "{}", String::from_utf8_lossy(&refused.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[ABI-3]` — `ember_module_init` checks the runtime's ABI version before anything else and
/// returns `EMBER_MODULE_INIT_ABI_MISMATCH` on a mismatch: the module's C, compiled against a
/// runtime header of the next version, linked with this runtime, is refused, and the runtime is
/// left untouched (an at-exit hook the host registered does not run at a shutdown, since nothing
/// was initialised).
#[test]
fn a_module_built_for_another_runtime_abi_is_refused_first() {
    let dir = scratch();
    let toolchain = toolchain();
    package(&dir.join("plugin"), "[package]\nname = \"plugin\"\nkind = \"cdylib\"\n",
        "@export(\"plugin_value\")\nfn plugin_value() -> i32:\n    return 3\n");
    let emitted = ember(&["build", "--emit", "c", "--out-dir", "out"], &dir.join("plugin"));
    assert_success(&emitted, "the module's C");
    let module_c = dir.join("plugin/out/debug/c/lib.c");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).unwrap();
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    // The next version's header: the number moved on, its own check of the number gone.
    let header_name = ember_branding::runtime_header();
    let current = std::fs::read_to_string(runtime.join("include").join(&header_name)).unwrap().replace("\r\n", "\n");
    let mut next = String::new();
    let mut skip = false;
    for line in current.lines() {
        if line.starts_with("#if EMBER_RUNTIME_ABI != ") { skip = true; continue; }
        if skip { if line.starts_with("#endif") { skip = false; } continue; }
        next.push_str(&line.replace("#define EMBER_RUNTIME_ABI 2", "#define EMBER_RUNTIME_ABI 3"));
        next.push('\n');
    }
    assert!(next.contains("#define EMBER_RUNTIME_ABI 3"), "the runtime header's version line moved");
    let next_include = dir.join("next");
    std::fs::create_dir_all(&next_include).unwrap();
    std::fs::write(next_include.join(&header_name), next).unwrap();
    std::fs::copy(runtime.join("include").join(format!("{}_simd.h", ember_branding::SYMBOL_PREFIX)),
        next_include.join(format!("{}_simd.h", ember_branding::SYMBOL_PREFIX))).unwrap();
    let ext = if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    let module_object = dir.join(format!("module.{ext}"));
    ember_build::compile_object(&toolchain, &module_c, &[next_include], &module_object, Profile::Debug)
        .expect("the module compiles against the next header");
    let host = dir.join("host.c");
    std::fs::write(&host, branded(&format!("#include \"{header_name}\"\n#include <stdio.h>\nint @RT@module_init(const @RT@rt_config* cfg);\nstatic int g_hooked;\nstatic void hook(void* data) {{ (void)data; g_hooked = 1; }}\nint main(void) {{\n    @RT@rt_config cfg = @RT@rt_config_default();\n    if (@RT@rt_at_exit(hook, NULL) != 0) return 2;\n    if (@RT@module_init(&cfg) != EMBER_MODULE_INIT_ABI_MISMATCH) return 3;\n    @RT@rt_shutdown();\n    if (g_hooked) return 4;\n    printf(\"ok\\n\");\n    return 0;\n}}\n"))).unwrap();
    let exe = dir.join(if cfg!(windows) { "host.exe" } else { "host" });
    std::fs::create_dir_all(dir.join("obj")).unwrap();
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[host, module_object, runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
        include_dirs: &[runtime.join("include")], output: exe.clone(), profile: Profile::Debug, obj_dir: dir.join("obj"),
    }).expect("the host links the module and this runtime");
    let output = run_host(&exe, &[], &dir);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok", "{:?} {}", output.status.code(),
        String::from_utf8_lossy(&output.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[FFI-26]` — a host that loads a library through one entry point finds its `@export_table` by
/// name, checks its protocol version, and calls through it. `[FFI-33c]`: each entry keeps its own
/// thread contract, its function's: a `main` entry called on another thread panics, an `any`
/// entry beside it does not, in every profile; the header names each entry's contract.
#[test]
fn an_export_table_is_found_by_name_with_threads_per_entry() {
    let dir = scratch();
    let toolchain = toolchain();
    package(&dir.join("game"), "[package]\nname = \"game\"\nkind = \"cdylib\"\n",
        "@export(threads=main)\npub extern \"C\" fn on_draw() -> i32:\n    return 5\n\n\
         pub extern \"C\" fn on_tick(n: i32) -> i32:\n    return n + 1\n\n\
         @export_table(\"GameApi\", protocol=3)\nstruct GameApi:\n    on_draw: extern \"C\" fn() -> i32\n    on_tick: extern \"C\" fn(i32) -> i32\n");
    for profile in ["debug", "release", "shipping"] {
        let built = ember(&["build", "--profile", profile], &dir.join("game"));
        assert_success(&built, "the cdylib with a table");
        let lib = dir.join("game/target").join(profile).join("lib");
        let header = std::fs::read_to_string(lib.join("game.h")).unwrap();
        for text in ["/*   on_draw: main */", "/*   on_tick: any */", "#define GameApi_PROTOCOL 3u", "extern const uint32_t GameApi_protocol_3;"] {
            assert!(header.contains(text), "the header lacks `{text}`:\n{header}");
        }
        let path = library_file(&lib, "game").to_string_lossy().replace('\\', "\\\\");
        // A library's types are named in C under its package's namespace.
        let package_bytes = "game".bytes().map(|byte| format!("{byte:02x}")).collect::<String>();
        let table_type = format!("{}pkg_{package_bytes}_{}", ember_branding::mangle_prefix(), ember_branding::mangled("GameApi"));
        assert!(header.contains(&format!("extern const {table_type} GameApi;")), "{header}");
        let host = format!(r#"{HOST_PRELUDE}
#include "game.h"
static const {table_type}* g_api;
static const char* g_mode;
#if defined(_WIN32)
static DWORD WINAPI other(LPVOID unused) {{
#else
static void* other(void* unused) {{
#endif
    (void)unused;
    if (strcmp(g_mode, "main") == 0) {{
        g_api->on_draw();
    }} else {{
        printf("%d\n", g_api->on_tick(41));
    }}
    return 0;
}}
int main(int argc, char** argv) {{
    g_mode = argc > 1 ? argv[1] : "any";
    @RT@rt_config cfg = @RT@rt_config_default();
    cfg.on_panic = host_panic;
    library l = load("{path}");
    if (!l) return 2;
    module_init_fn init = (module_init_fn)find(l, "@RT@module_init");
    g_api = (const {table_type}*)find(l, "GameApi");
    const uint32_t* protocol = (const uint32_t*)find(l, "GameApi_protocol_3");
    if (!init || !g_api || !protocol || *protocol != GameApi_PROTOCOL) return 3;
    if (init(&cfg) != 0) return 4;
    if (g_api->on_draw() != 5 || g_api->on_tick(1) != 2) return 5;
#if defined(_WIN32)
    HANDLE t = CreateThread(NULL, 0, other, NULL, 0, NULL);
    WaitForSingleObject(t, INFINITE);
#else
    pthread_t t;
    pthread_create(&t, NULL, other, NULL);
    pthread_join(t, NULL);
#endif
    return 0;
}}
    "#);
        let exe = link_host(&toolchain, &dir, &lib, &format!("host_{profile}"), &host);
        let any = run_host(&exe, &["any"], &dir);
        assert_eq!(String::from_utf8_lossy(&any.stdout).trim(), "42", "{profile}: {:?} {}", any.status.code(), String::from_utf8_lossy(&any.stderr));
        assert!(any.status.success());
        let main = run_host(&exe, &["main"], &dir);
        assert!(String::from_utf8_lossy(&main.stdout).contains("panic: export requires the module initialization thread"),
            "{profile}: {:?} {}{}", main.status.code(), String::from_utf8_lossy(&main.stdout), String::from_utf8_lossy(&main.stderr));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[FFI-31]` — shutting a module down ends the threads it started for jobs (each finishes and is
/// joined), so none runs the library's code once the host unloads it: the process has as many
/// threads after `ember_module_shutdown` as before `ember_module_init`.
#[test]
fn shutdown_ends_the_job_workers_a_module_started() {
    let dir = scratch();
    let toolchain = toolchain();
    package(&dir.join("worker"), "[package]\nname = \"worker\"\nkind = \"cdylib\"\n",
        "import std.jobs\nfrom std.sync import Atomic\n\nstatic TOTAL: Atomic[int] = Atomic(0)\n\n\
         @export(\"busy\")\nfn busy() -> i32:\n    jobs.wait(jobs.submit(owned fn() => TOTAL.fetch_add(1)))\n    return TOTAL.load() as i32\n");
    let built = ember(&["build"], &dir.join("worker"));
    assert_success(&built, "the cdylib running jobs");
    let lib = dir.join("worker/target/debug/lib");
    let path = library_file(&lib, "worker").to_string_lossy().replace('\\', "\\\\");
    let host = format!(r#"{HOST_PRELUDE}
#if defined(_WIN32)
#include <tlhelp32.h>
static int threads(void) {{
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    THREADENTRY32 entry;
    entry.dwSize = sizeof entry;
    int count = 0;
    for (BOOL more = Thread32First(snapshot, &entry); more; more = Thread32Next(snapshot, &entry)) {{
        count += entry.th32OwnerProcessID == GetCurrentProcessId();
    }}
    CloseHandle(snapshot);
    return count;
}}
#else
#include <dirent.h>
static int threads(void) {{
    DIR* dir = opendir("/proc/self/task");
    int count = 0;
    for (struct dirent* entry = readdir(dir); entry; entry = readdir(dir)) {{
        count += entry->d_name[0] != '.';
    }}
    closedir(dir);
    return count;
}}
#endif
int main(void) {{
    ember_rt_config cfg = ember_rt_config_default();
    library l = load("{path}");
    if (!l) return 2;
    module_init_fn init = (module_init_fn)find(l, "@RT@module_init");
    module_shutdown_fn shutdown = (module_shutdown_fn)find(l, "@RT@module_shutdown");
    value_fn busy = (value_fn)find(l, "busy");
    if (!init || !shutdown || !busy) return 3;
    int before = threads();
    if (init(&cfg) != 0 || busy() != 1) return 4;
    int working = threads();
    shutdown();
    int after = threads();
    unload(l);
    printf("%d %d %d\n", before, working > before, after);
    return after == before && working > before ? 0 : 5;
}}
"#);
    let exe = link_host(&toolchain, &dir, &lib, "host", &host);
    let ran = run_host(&exe, &[], &dir);
    assert!(ran.status.success(), "the workers outlived the shutdown ({:?}): {}{}", ran.status.code(),
        String::from_utf8_lossy(&ran.stdout), String::from_utf8_lossy(&ran.stderr));
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[BLD-FFI-3]` — `[link] libs` links existing libraries by path and by name, in the order
/// written: two archives both define `which_lib`, and the linker takes it from the first one the
/// list names, so swapping them swaps the answer. A name is searched for where the C toolchain
/// searches: `shlwapi` from the Windows SDK; elsewhere `counter`, an archive on `LIBRARY_PATH`.
#[test]
fn libraries_link_by_path_and_name_in_declaration_order() {
    let dir = scratch();
    let toolchain = toolchain();
    let vendor = dir.join("app/vendor");
    std::fs::create_dir_all(&vendor).unwrap();
    let ext = if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    let archive = |name: &str, source: &str| {
        let c = vendor.join(format!("{name}.c"));
        std::fs::write(&c, source).unwrap();
        let object = vendor.join(format!("{name}.{ext}"));
        ember_build::compile_object(&toolchain, &c, &[], &object, Profile::Debug).expect("the library's C compiles");
        let file = vendor.join(ember_build::archive_name(&toolchain, name));
        ember_build::archive_objects(&toolchain, &[object], &file).expect("the archive is made");
        file.file_name().unwrap().to_string_lossy().into_owned()
    };
    let one = archive("one", "int which_lib(void) { return 1; }\n");
    let two = archive("two", "int which_lib(void) { return 2; }\n");
    archive("counter", "int counter_start(void) { return 40; }\n");
    let (named, declaration, call) = if cfg!(windows) {
        ("shlwapi", "unsafe extern \"system\":\n    fn StrToIntA(text: cstr) -> i32\n",
            "    # SAFETY(ffi): StrToIntA only reads the NUL-terminated literal.\n    unsafe:\n        println(StrToIntA(c\"42\"))\n")
    } else {
        ("counter", "unsafe extern \"C\":\n    safe fn counter_start() -> i32\n", "    println(counter_start() + 2)\n")
    };
    let source = format!("unsafe extern \"C\":\n    safe fn which_lib() -> i32\n\n{declaration}\nfn main():\n    println(which_lib())\n{call}");
    let mut outputs = Vec::new();
    for (first, second) in [(&one, &two), (&two, &one)] {
        package(&dir.join("app"), &format!(
            "[package]\nname = \"app\"\nkind = \"bin\"\n\n[link]\nlibs = [\"vendor/{first}\", \"vendor/{second}\", \"{named}\"]\n"),
            "");
        std::fs::remove_file(dir.join("app/src").join(ember_branding::source_file("lib"))).unwrap();
        std::fs::write(dir.join("app/src").join(ember_branding::source_file("main")), &source).unwrap();
        let built = Command::new(EMBER).args(["build"]).current_dir(dir.join("app"))
            .env("LIBRARY_PATH", &vendor).output().expect("ember starts");
        assert_success(&built, "the build linking libraries");
        let exe = dir.join("app/target/debug/bin").join(if cfg!(windows) { "main.exe" } else { "main" });
        let ran = run_host(&exe, &[], &dir);
        assert!(ran.status.success(), "{}", String::from_utf8_lossy(&ran.stderr));
        outputs.push(String::from_utf8_lossy(&ran.stdout).replace("\r\n", "\n"));
    }
    assert_eq!(outputs, ["1\n42\n", "2\n42\n"]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The names a PE image exports, from its export directory.
fn pe_exports(image: &[u8]) -> Vec<String> {
    let u16_at = |at: usize| u16::from_le_bytes([image[at], image[at + 1]]) as usize;
    let u32_at = |at: usize| u32::from_le_bytes([image[at], image[at + 1], image[at + 2], image[at + 3]]) as usize;
    let pe = u32_at(0x3c);
    let sections = u16_at(pe + 6);
    let optional = pe + 24;
    let optional_size = u16_at(pe + 20);
    let directories = optional + if u16_at(optional) == 0x20b { 112 } else { 96 };
    let export_rva = u32_at(directories);
    let table = optional + optional_size;
    let offset = |rva: usize| -> usize {
        (0..sections).map(|index| table + index * 40).find_map(|section| {
            let (virtual_address, size, raw) = (u32_at(section + 12), u32_at(section + 16).max(u32_at(section + 8)), u32_at(section + 20));
            (rva >= virtual_address && rva < virtual_address + size).then(|| rva - virtual_address + raw)
        }).expect("an RVA inside a section")
    };
    let directory = offset(export_rva);
    let (count, names) = (u32_at(directory + 24), offset(u32_at(directory + 32)));
    (0..count).map(|index| {
        let name = offset(u32_at(names + index * 4));
        let end = image[name..].iter().position(|&byte| byte == 0).unwrap();
        String::from_utf8_lossy(&image[name..name + end]).into_owned()
    }).collect()
}
