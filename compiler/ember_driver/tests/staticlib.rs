//! Separate C and C++ translation units consume the shipped library artifacts.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use ember_build::{LinkRequest, Profile, Toolchain};

const EMBER: &str = env!("CARGO_BIN_EXE_ember");
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let path = std::env::temp_dir().join(format!("{}-staticlib-{}-{}",
        ember_branding::CLI_NAME, std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&path).expect("scratch directory");
    path
}

fn build(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(EMBER).args(args).current_dir(cwd).output().expect("ember starts")
}

fn assert_success(output: &std::process::Output, step: &str) {
    assert!(output.status.success(), "{step} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

fn public_type_name(package: &str, name: &str) -> String {
    let bytes = package.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    format!("{}pkg_{bytes}_{}", ember_branding::mangle_prefix(), ember_branding::mangled(name))
}

fn compile_cpp_object(toolchain: &Toolchain, source: &Path, object: &Path, include: &Path) {
    let mut command = match toolchain {
        Toolchain::Msvc { cl, env } => {
            let mut command = Command::new(cl);
            command.envs(env).args(["/nologo", "/TP", "/std:c++17", "/MD", "/c"]);
            command.arg(format!("/I{}", include.display()));
            command.arg(source).arg(format!("/Fo{}", object.display()));
            command
        }
        Toolchain::Clang(path) | Toolchain::Gcc(path) => {
            let mut command = Command::new(path);
            command.args(["-x", "c++", "-std=c++17", "-Wall", "-Wextra"]);
            command.arg("-I").arg(include);
            command.arg("-c").arg(source).arg("-o").arg(object);
            command
        }
    };
    let result = command.output().expect("C++ compiler starts");
    assert_success(&result, "C++ host object");
}

#[test]
fn staticlib_archives_and_headers_link_from_separate_c_and_cpp_hosts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).unwrap();
    let scratch = scratch();
    let package = scratch.join("first");
    let second = scratch.join("second");
    let runtime_stem = format!("{}_rt", ember_branding::SYMBOL_PREFIX);
    let runtime_header_name = ember_branding::runtime_header();
    for dir in [&package, &second] { std::fs::create_dir_all(dir.join("src")).unwrap(); }
    std::fs::write(package.join(ember_branding::MANIFEST), format!(
        "[package]\nname = \"{runtime_stem}\"\nkind = \"staticlib\"\n[build]\nruntime = \"static\"\n")).unwrap();
    let fixture = root.join("tests/conformance/FFI-21")
        .join(ember_branding::source_file("accept_export_header_api"));
    let mut source = std::fs::read_to_string(fixture).unwrap();
    source.push_str("\nfn main():\n    println(0)\n\nfn private_library_bias(value: i32) -> i32:\n    return value + 11\n\npub extern \"C\" fn first_library_value() -> i32:\n    return private_library_bias(0)\n");
    std::fs::write(package.join("src").join(ember_branding::source_file("lib")), source).unwrap();
    std::fs::write(second.join(ember_branding::MANIFEST),
        "[package]\nname = \"second_api\"\nkind = \"staticlib\"\n").unwrap();
    std::fs::write(second.join("src").join(ember_branding::source_file("lib")),
        "struct HeaderLeaf:\n    value: i32\n    extra: i32\n\nfn private_library_bias(value: i32) -> i32:\n    return value\n\npub extern \"C\" fn second_value(leaf: HeaderLeaf, callback: Option[extern \"C\" fn(i32) -> i32]) -> i32:\n    match callback:\n        Some(function):\n            return function(private_library_bias(leaf.value) + leaf.extra)\n        None:\n            return private_library_bias(leaf.value) + leaf.extra\n").unwrap();

    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C toolchain available");
    let out = scratch.join("out");
    let out_arg = out.to_string_lossy().into_owned();
    let package_arg = package.to_string_lossy().into_owned();
    let second_arg = second.to_string_lossy().into_owned();
    let header_only_out = scratch.join("header-only");
    let header_only_arg = header_only_out.to_string_lossy().into_owned();
    let header_only = Command::new(EMBER).args(["build", package_arg.as_str(), "--emit", "header",
        "--out-dir", header_only_arg.as_str()]).current_dir(&scratch)
        .env(ember_branding::cc_var(), format!("{}-compiler-that-does-not-exist", ember_branding::CLI_NAME))
        .output().expect("header-only compiler starts");
    assert_success(&header_only, "collision-safe header-only emit");
    let header_only_lib = header_only_out.join("debug/lib");
    assert!(header_only_lib.join(&runtime_header_name).is_file());
    assert!(header_only_lib.join(format!("{}_runtime/{}",
        ember_branding::SYMBOL_PREFIX, ember_branding::runtime_header())).is_file());
    let header_caller = scratch.join("header-only-caller.c");
    std::fs::write(&header_caller, format!(
        "#include \"{runtime_header_name}\"\n{wide} call_export({wide} value) {{ return header_i128_identity(value); }}\n",
        wide = ember_branding::runtime("i128"),
    )).unwrap();
    let header_caller_object = scratch.join(if cfg!(windows) { "header-only-caller.obj" } else { "header-only-caller.o" });
    ember_build::compile_object(&toolchain, &header_caller, &[header_only_lib],
        &header_caller_object, Profile::Debug)
        .expect("a C caller compiles against the header-only artifact named like the runtime");
    let c_only = Command::new(EMBER).args(["build", package_arg.as_str(), "--emit", "c",
        "--out-dir", header_only_arg.as_str()]).current_dir(&scratch)
        .env(ember_branding::cc_var(), format!("{}-compiler-that-does-not-exist", ember_branding::CLI_NAME))
        .output().expect("C-only compiler starts");
    assert_success(&c_only, "staticlib C-only emit");
    let c_only_source = std::fs::read_to_string(header_only_out.join("debug/c/lib.c")).unwrap();
    let header_only_source = std::fs::read_to_string(header_only_out.join("debug/lib").join(&runtime_header_name)).unwrap();
    for name in [public_type_name(&runtime_stem, "HeaderLeaf"),
        public_type_name(&runtime_stem, "HeaderEnvelope")] {
        assert!(c_only_source.contains(&name) && header_only_source.contains(&name),
            "C-only output and package header disagree on public type {name}");
    }

    for profile in [Profile::Debug, Profile::Release, Profile::Shipping] {
        let profile_name = profile.name();
        let mut args = vec!["build", "--profile", profile_name, "--out-dir", out_arg.as_str()];
        if profile != Profile::Debug { args.push(package_arg.as_str()); }
        let build_first = build(&args, &package);
        assert_success(&build_first, "first staticlib build");
        let lib = out.join(profile_name).join("lib");
        let runtime_lib = lib.join(format!("{}_runtime", ember_branding::SYMBOL_PREFIX));
        let package_archive = lib.join(ember_build::archive_name(&toolchain, &runtime_stem));
        let runtime_archive = runtime_lib.join(ember_build::archive_name(&toolchain,
            &runtime_stem));
        let header = lib.join(&runtime_header_name);
        let support_header = runtime_lib.join(&runtime_header_name);
        for artifact in [&package_archive, &runtime_archive, &header, &support_header] {
            assert!(artifact.is_file(), "missing distributable {}", artifact.display());
        }
        assert_ne!(package_archive, runtime_archive, "package/runtime archives collided");
        assert_ne!(header, support_header, "package/runtime headers collided");
        let header_text = std::fs::read_to_string(&header).unwrap();
        assert!(header_text.contains(&format!("{}_runtime/{}",
            ember_branding::SYMBOL_PREFIX, ember_branding::runtime_header())));
        let generated_c = std::fs::read_to_string(out.join(profile_name).join("c/lib.c")).unwrap();
        assert!(!generated_c.contains("int main("), "staticlib emitted a process entry point");

        let c_host = scratch.join(format!("host-{profile_name}.c"));
        let leaf = public_type_name(&runtime_stem, "HeaderLeaf");
        let envelope = public_type_name(&runtime_stem, "HeaderEnvelope");
        let rt_init = ember_branding::runtime("rt_init");
        let rt_shutdown = ember_branding::runtime("rt_shutdown");
        let wide = ember_branding::runtime("i128");
        let wide_make = ember_branding::runtime("i128_make");
        let wide_eq = ember_branding::runtime("i128_eq");
        std::fs::write(&c_host, format!(
            "#include \"{runtime_header_name}\"\nstatic int32_t plus_one(int32_t value) {{ return value + 1; }}\nint main(void) {{\n    {rt_init}(NULL);\n    {leaf} leaf = {{19}};\n    {envelope} value = {{leaf, 22}};\n    if (header_score(value, plus_one) != 42) return 1;\n    {wide} x = {wide_make}(0, 42);\n    if (!{wide_eq}(header_i128_identity(x), x)) return 2;\n    if (header_creator() != 46 || header_main() != 48) return 3;\n    {rt_shutdown}();\n    return 0;\n}}\n"
        )).unwrap();
        let executable = scratch.join(if cfg!(windows) {
            format!("host-{profile_name}.exe")
        } else { format!("host-{profile_name}") });
        let objects = scratch.join(format!("host-objects-{profile_name}"));
        std::fs::create_dir_all(&objects).unwrap();
        ember_build::compile_and_link(&toolchain, &LinkRequest {
            sources: &[c_host, package_archive.clone(), runtime_archive.clone()],
            include_dirs: &[lib.clone()], output: executable.clone(), profile,
            obj_dir: objects,
        }).expect("ordinary C host links package and runtime archives");
        assert_success(&Command::new(&executable).output().expect("C host runs"), "C host");

        if profile == Profile::Debug {
            let second_build = build(&["build", second_arg.as_str(), "--out-dir", out_arg.as_str()], root);
            assert_success(&second_build, "second staticlib build");
            let second_archive = lib.join(ember_build::archive_name(&toolchain, "second_api"));
            let combined_host = scratch.join("both.c");
            let second_leaf = public_type_name("second_api", "HeaderLeaf");
            std::fs::write(&combined_host, format!(
            "#include \"{runtime_header_name}\"\n#include \"second_api.h\"\nint main(void) {{ {rt_init}(NULL); {second_leaf} leaf = {{3, 4}}; int ok = header_main() == 48 && first_library_value() == 11 && second_value(leaf, NULL) == 7; {rt_shutdown}(); return ok ? 0 : 1; }}\n"
            )).unwrap();
            let combined = scratch.join(if cfg!(windows) { "both.exe" } else { "both" });
            let combined_obj = scratch.join("both-objects");
            std::fs::create_dir_all(&combined_obj).unwrap();
            ember_build::compile_and_link(&toolchain, &LinkRequest {
                sources: &[combined_host, package_archive.clone(), second_archive.clone(), runtime_archive.clone()],
                include_dirs: &[lib.clone()], output: combined.clone(), profile,
                obj_dir: combined_obj,
            }).expect("two Ember package archives link without private-symbol collisions");
            assert_success(&Command::new(combined).output().expect("two-library host runs"), "two-library host");

            let cpp_host = scratch.join("host.cpp");
            std::fs::write(&cpp_host, format!(
                "#include \"{runtime_header_name}\"\n#include \"second_api.h\"\nint main() {{ {rt_init}(nullptr); {second_leaf} leaf{{3, 4}}; int ok = header_main() == 48 && first_library_value() == 11 && second_value(leaf, nullptr) == 7; {rt_shutdown}(); return ok ? 0 : 1; }}\n"
            )).unwrap();
            let cpp_object = scratch.join(if cfg!(windows) { "host_cpp.obj" } else { "host_cpp.o" });
            compile_cpp_object(&toolchain, &cpp_host, &cpp_object, &lib);
            let cpp_exe = scratch.join(if cfg!(windows) { "host_cpp.exe" } else { "host_cpp" });
            let cpp_obj_dir = scratch.join("cpp-objects");
            std::fs::create_dir_all(&cpp_obj_dir).unwrap();
            ember_build::compile_and_link(&toolchain, &LinkRequest {
                sources: &[cpp_object, package_archive, second_archive, runtime_archive],
                include_dirs: &[lib], output: cpp_exe.clone(), profile,
                obj_dir: cpp_obj_dir,
            }).expect("ordinary C++ host links package and runtime archives");
            assert_success(&Command::new(cpp_exe).output().expect("C++ host runs"), "C++ host");
        }
    }
}

/// `[CG-C-11]` — a static library with a `@fastmath` or `@fp(contract)`
/// function is refused rather than built with the attribute lost: its
/// relaxed unit would call functions the library keeps internal to its main
/// unit (`docs/NOT-IMPLEMENTED.md`).
#[test]
fn a_static_library_with_a_relaxed_function_is_refused() {
    let dir = scratch();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src").join(ember_branding::source_file("lib")),
        "@fastmath
fn scaled(x: f64) -> f64:
    return x * 2.0

pub extern \"C\" fn relaxed_value(x: f64) -> f64:
    return scaled(x)
").unwrap();
    std::fs::write(dir.join(ember_branding::MANIFEST),
        "[package]
name = \"relaxed_api\"
kind = \"staticlib\"
").unwrap();
    let output = build(&["build"], &dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "a static library with a relaxed function was built:
{stderr}");
    assert!(stderr.contains("error[E0900]: `@fastmath` and `@fp(contract)` in a static library are not built yet"), "{stderr}");
}

#[test]
fn requested_unsupported_package_modes_fail_before_linking() {
    let dir = scratch();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let source = dir.join("src").join(ember_branding::source_file("lib"));
    std::fs::write(&source, "pub extern \"C\" fn answer() -> i32:\n    return 42\n").unwrap();
    let manifest = dir.join(ember_branding::MANIFEST);
    std::fs::write(&manifest,
        "[package]\nname = \"mode_check\"\nkind = \"staticlib\"\n[build]\nruntime = \"shared\"\n").unwrap();
    let shared = build(&["build"], &dir);
    assert!(!shared.status.success());
    assert!(String::from_utf8_lossy(&shared.stderr).contains("runtime mode `shared`"));

    std::fs::write(&manifest,
        "[package]\nname = \"mode_check\"\nkind = \"cdylib\"\n[build]\nruntime = \"static\"\n").unwrap();
    let cdylib = build(&["build"], &dir);
    assert!(!cdylib.status.success());
    assert!(String::from_utf8_lossy(&cdylib.stderr).contains("package kind `cdylib`"));

    std::fs::write(&manifest,
        "[package]\nname = \"mode_check\"\nkind = \"bin\"\n[build]\nruntime = \"shared\"\n").unwrap();
    let source_arg = source.to_string_lossy().into_owned();
    let run = build(&["run", source_arg.as_str()], &dir);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("runtime mode `shared`"));
}

#[test]
fn manifest_entry_below_src_resolves_sibling_imports_from_package_root() {
    let package = scratch();
    let sub = package.join("src").join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let entry_relative = Path::new("src").join("sub").join(ember_branding::source_file("entry"));
    let entry_setting = entry_relative.to_string_lossy().replace('\\', "/");
    std::fs::write(package.join(ember_branding::MANIFEST), format!(
        "[package]\nname = \"entry_package\"\nkind = \"bin\"\n[build]\nentry = \"{entry_setting}\"\n")).unwrap();
    let entry = package.join(entry_relative);
    std::fs::write(&entry,
        "import sub.helper as helper\n\nfn main():\n    println(helper.answer())\n").unwrap();
    std::fs::write(sub.join(ember_branding::source_file("helper")),
        "pub fn answer() -> i32:\n    return 42\n").unwrap();
    let out = package.join("out");
    let out_arg = out.to_string_lossy().into_owned();
    let no_input = build(&["build", "--out-dir", out_arg.as_str()], &sub);
    assert_success(&no_input, "manifest build from nested directory");
    let executable = out.join("debug/bin").join(if cfg!(windows) { "entry.exe" } else { "entry" });
    let ran = Command::new(&executable).output().expect("nested entry executable runs");
    assert_success(&ran, "nested entry executable");
    assert_eq!(String::from_utf8_lossy(&ran.stdout).replace("\r\n", "\n"), "42\n");

    let entry_arg = entry.to_string_lossy().into_owned();
    let explicit = Command::new(EMBER).args(["build", entry_arg.as_str(), "--emit", "c"])
        .current_dir(&package)
        .env(ember_branding::cc_var(), format!("{}-compiler-that-does-not-exist", ember_branding::CLI_NAME))
        .output().expect("explicit entry C-only compiler starts");
    assert_success(&explicit, "explicit nested source invocation");
}
