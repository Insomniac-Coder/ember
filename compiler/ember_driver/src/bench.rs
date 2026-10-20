//! `ember bench [filter] [--compare]` — the performance gate (`[TST-28]`, `[BEN-1]`..`[BEN-6]`).
//!
//! `tests/perf/<name>.em` is a benchmark program and `tests/perf/<name>.c` its equivalent C program
//! (`.cpp` for counted-object code, `[BEN-6]`). The `.em` file's `#$ perf: <category>` annotation
//! gives the threshold; `#$ threshold: x` may only tighten it. Both programs must print the same
//! output. With `--compare`, each pair is built with one toolchain and the release profile, then
//! timed on one pinned core: 3 untimed and 30 timed repetitions of the Ember program, the C program
//! and a second copy of the C program, interleaved, so drift falls on all three alike.

#![allow(unsafe_code)] // pinning to a core and reading the power supply are system calls

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

/// `[BEN-1]` — the repetitions, untimed and timed.
const WARMUP: usize = 3;
const TIMED: usize = 30;
/// `[BEN-2]` — an inconclusive benchmark is run again, this many times in all.
const ATTEMPTS: usize = 3;
/// `[BEN-3]` — the band the C program's ratio to its own copy must reach.
const FIT_BAND: (f64, f64) = (0.98, 1.02);

/// `[BEN-6]` — what a benchmark measures, which fixes its threshold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Category {
    /// Scalar code and tight loops: within 1.05x C.
    Scalar,
    /// `SoA` and SIMD code: within 1.10x.
    Soa,
    /// Counted-object code against a C++ intrusive count: within 1.15x.
    Counted,
}

impl Category {
    fn parse(word: &str) -> Option<Category> {
        match word {
            "scalar" | "loop" => Some(Category::Scalar),
            "soa" | "simd" => Some(Category::Soa),
            "counted" => Some(Category::Counted),
            _ => None,
        }
    }

    pub(crate) fn threshold(self) -> f64 {
        match self {
            Category::Scalar => 1.05,
            Category::Soa => 1.10,
            Category::Counted => 1.15,
        }
    }
}

/// One side's timings: the median and the 95 % confidence interval of the median (`[BEN-1]`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Side {
    pub median: f64,
    pub low: f64,
    pub high: f64,
}

/// The median of `samples`.
pub(crate) fn median(samples: &[f64]) -> f64 {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    if n % 2 == 1 { sorted[n / 2] } else { (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0 }
}

/// The 1-based ranks `(j, n + 1 - j)` of the order statistics bounding the median with at least
/// 95 % confidence, distribution-free: the largest `j` with P(X(j) <= median <= X(n+1-j)) >= 0.95
/// under Binomial(n, 1/2).
pub(crate) fn median_ranks(n: usize) -> (usize, usize) {
    // P(j <= B <= n - j) for B ~ Binomial(n, 1/2), as a sum of terms built incrementally.
    let mut terms = vec![0.0f64; n + 1];
    let mut term = 0.5f64.powi(n as i32);
    for (i, slot) in terms.iter_mut().enumerate() {
        *slot = term;
        term = term * (n - i) as f64 / (i + 1) as f64;
    }
    let mut best = 1;
    for j in 1..=n / 2 {
        let covered: f64 = terms[j..=n - j].iter().sum();
        if covered >= 0.95 {
            best = j;
        } else {
            break;
        }
    }
    (best, n + 1 - best)
}

/// `[BEN-1]` — the median of `samples` and its 95 % interval.
pub(crate) fn side(samples: &[f64]) -> Side {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let (j, k) = median_ranks(sorted.len());
    Side { median: median(&sorted), low: sorted[j - 1], high: sorted[k - 1] }
}

/// `[BEN-1]` — the ratio of two sides, `a` over `b`: the medians' ratio and an interval holding
/// every ratio of values the two intervals allow.
pub(crate) fn ratio(a: Side, b: Side) -> Side {
    Side { median: a.median / b.median, low: a.low / b.high, high: a.high / b.low }
}

/// What a benchmark's run says.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Verdict {
    Pass,
    /// `[BEN-2]` — the lower bound of the ratio's interval exceeds the threshold.
    Fail(String),
    /// `[BEN-2]` — the point estimate exceeds the threshold, its interval includes it.
    Inconclusive,
    /// `[BEN-3]`, `[BEN-5]` — the run measures nothing: the machine or the builds are unfit.
    Void(String),
}

/// `[BEN-2]`, `[BEN-3]` — the verdict for Ember's ratio to C, given the C program's ratio to its
/// own copy.
pub(crate) fn verdict(ember_to_c: Side, c_to_itself: Side, threshold: f64) -> Verdict {
    if c_to_itself.high < FIT_BAND.0 || c_to_itself.low > FIT_BAND.1 {
        return Verdict::Void(format!(
            "the C program against its own copy is {:.3} [{:.3}, {:.3}], outside 1.00 +/- 0.02: the machine is unfit for gating",
            c_to_itself.median, c_to_itself.low, c_to_itself.high
        ));
    }
    if ember_to_c.low > threshold {
        return Verdict::Fail(format!("{:.3}x C, at least {:.3}x, above {threshold:.2}x", ember_to_c.median, ember_to_c.low));
    }
    if ember_to_c.median > threshold {
        return Verdict::Inconclusive;
    }
    Verdict::Pass
}

/// One benchmark pair of `tests/perf/`.
struct Pair {
    name: String,
    ember: PathBuf,
    twin: PathBuf,
    category: Category,
    threshold: f64,
    stdin: Option<String>,
}

/// The pairs under `dir` whose names contain `filter`; an `.em` file with no twin, or with no
/// `#$ perf:` category, is an error.
fn pairs(dir: &Path, filter: Option<&str>) -> Result<Vec<Pair>, String> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    let mut paths: Vec<PathBuf> = entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect();
    paths.sort();
    for path in paths {
        if path.extension().and_then(|ext| ext.to_str()) != Some(ember_branding::SOURCE_EXT) {
            continue;
        }
        let name = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default().to_string();
        if filter.is_some_and(|filter| !name.contains(filter)) {
            continue;
        }
        let source = std::fs::read_to_string(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let annotation = |key: &str| {
            source.lines().find_map(|line| line.strip_prefix(&format!("#$ {key}:")).map(|value| value.trim().to_string()))
        };
        let category = annotation("perf")
            .ok_or_else(|| format!("{}: no `#$ perf: scalar | soa | counted` annotation [BEN-6]", path.display()))?;
        let category = Category::parse(&category)
            .ok_or_else(|| format!("{}: `{category}` is not a benchmark category (scalar, loop, soa, simd, counted) [BEN-6]", path.display()))?;
        let mut threshold = category.threshold();
        if let Some(written) = annotation("threshold") {
            let written: f64 = written.parse().map_err(|_| format!("{}: `#$ threshold: {written}` is not a number", path.display()))?;
            if written > threshold {
                return Err(format!("{}: a threshold may only tighten its category's {threshold:.2}x [BEN-6]", path.display()));
            }
            threshold = written;
        }
        let twin_ext = if category == Category::Counted { "cpp" } else { "c" };
        let twin = path.with_extension(twin_ext);
        if !twin.exists() {
            return Err(format!("{} has no equivalent program {} [TST-28]", path.display(), twin.display()));
        }
        found.push(Pair { name, ember: path, twin, category, threshold, stdin: annotation("stdin") });
    }
    Ok(found)
}

/// `[BEN-1]` — this process, and every program it starts, on the one core `core`.
fn pin_to_core(core: usize) -> Result<(), String> {
    #[cfg(windows)]
    {
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn SetProcessAffinityMask(process: *mut std::ffi::c_void, mask: usize) -> i32;
        }
        if core >= usize::BITS as usize {
            return Err(format!("core {core} is out of range"));
        }
        // SAFETY(ffi): the pseudo-handle of this process and a mask of one core.
        let pinned = unsafe { SetProcessAffinityMask(GetCurrentProcess(), 1usize << core) };
        if pinned == 0 {
            return Err(format!("cannot pin to core {core}: {}", std::io::Error::last_os_error()));
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        unsafe extern "C" {
            fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
        }
        let mut mask = [0u64; 16];
        if core >= mask.len() * 64 {
            return Err(format!("core {core} is out of range"));
        }
        mask[core / 64] |= 1u64 << (core % 64);
        // SAFETY(ffi): this process (0) and a `cpu_set_t`-sized mask of one core.
        let pinned = unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) };
        if pinned != 0 {
            return Err(format!("cannot pin to core {core}: {}", std::io::Error::last_os_error()));
        }
        Ok(())
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = core;
        Err("pinning to one core is not supported on this host".to_string())
    }
}

/// `[BEN-3]` — a machine running on battery is unfit for gating: its clocks follow the battery.
fn on_battery() -> bool {
    #[cfg(windows)]
    {
        #[repr(C)]
        #[derive(Default)]
        struct SystemPowerStatus {
            ac_line_status: u8,
            battery_flag: u8,
            battery_life_percent: u8,
            system_status_flag: u8,
            battery_life_time: u32,
            battery_full_life_time: u32,
        }
        unsafe extern "system" {
            fn GetSystemPowerStatus(status: *mut SystemPowerStatus) -> i32;
        }
        let mut status = SystemPowerStatus::default();
        // SAFETY(ffi): a status block the call fills.
        let read = unsafe { GetSystemPowerStatus(&mut status) };
        read != 0 && status.ac_line_status == 0
    }
    #[cfg(not(windows))]
    {
        // Linux: a mains supply that reports itself offline.
        std::fs::read_dir("/sys/class/power_supply").ok().is_some_and(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                let path = entry.path();
                std::fs::read_to_string(path.join("type")).is_ok_and(|kind| kind.trim() == "Mains")
                    && std::fs::read_to_string(path.join("online")).is_ok_and(|online| online.trim() == "0")
            })
        })
    }
}

/// The program at `exe`, run once: its standard output, and the time it took in milliseconds.
fn run_once(exe: &Path, stdin: Option<&str>) -> Result<(String, f64), String> {
    let started = Instant::now();
    let mut child = Command::new(exe)
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("cannot run {}: {error}", exe.display()))?;
    if let (Some(text), Some(mut input)) = (stdin, child.stdin.take()) {
        use std::io::Write;
        let _ = input.write_all(format!("{text}\n").as_bytes());
    }
    let output = child.wait_with_output().map_err(|error| format!("{}: {error}", exe.display()))?;
    let millis = started.elapsed().as_secs_f64() * 1000.0;
    if !output.status.success() {
        return Err(format!("{} exited with {}", exe.display(), output.status));
    }
    Ok((String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"), millis))
}

/// The Ember program, built for release by this compiler: the path it printed.
fn build_ember(source: &Path, out_dir: &Path) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let output = Command::new(exe)
        .args(["build", "--profile", "release"])
        .arg(source)
        .arg("--out-dir")
        .arg(out_dir)
        .output()
        .map_err(|error| error.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(format!("{} does not build:\n{}{}", source.display(), stdout, String::from_utf8_lossy(&output.stderr)));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let built = stdout
        .lines()
        .chain(stderr.lines())
        .find_map(|line| line.trim().strip_prefix("built "))
        .ok_or("the build printed no program")?;
    let built = PathBuf::from(built);
    Ok(if built.is_absolute() { built } else { std::env::current_dir().map_err(|error| error.to_string())?.join(built) })
}

/// `[BEN-5]` — the C program, built by the same toolchain and profile as the Ember program's C,
/// so both have one optimisation level, LTO setting, floating-point model and target-CPU flags.
fn build_twin(toolchain: &ember_build::Toolchain, source: &Path, out_dir: &Path, name: &str) -> Result<PathBuf, String> {
    let output = out_dir.join(format!("{name}_c{}", std::env::consts::EXE_SUFFIX));
    let obj_dir = out_dir.join(format!("{name}_c_obj"));
    std::fs::create_dir_all(&obj_dir).map_err(|error| error.to_string())?;
    let request = ember_build::LinkRequest {
        sources: &[source.to_path_buf()],
        include_dirs: &[],
        output: output.clone(),
        profile: ember_build::Profile::Release,
        obj_dir,
    };
    ember_build::compile_and_link(toolchain, &request).map_err(|error| format!("{} does not build: {error}", source.display()))?;
    Ok(output)
}

pub fn command(args: &[String]) -> Result<ExitCode, String> {
    let mut filter = None;
    let mut compare = false;
    let mut core = 1usize;
    let mut dir = PathBuf::from("tests/perf");
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--compare" => compare = true,
            "--core" => {
                let value = rest.next().ok_or("`--core` needs a core number")?;
                core = value.parse().map_err(|_| format!("`--core {value}` is not a core number"))?;
            }
            "--dir" => dir = PathBuf::from(rest.next().ok_or("`--dir` needs a folder")?),
            other if other.starts_with('-') => return Err(format!("unknown `ember bench` option `{other}`")),
            other => filter = Some(other.to_string()),
        }
    }
    if !compare {
        return Err("`ember bench` times programs against their C twins: run `ember bench --compare` [TST-28]".to_string());
    }
    let pairs = pairs(&dir, filter.as_deref())?;
    if pairs.is_empty() {
        println!("no benchmarks in {}", dir.display());
        return Ok(ExitCode::SUCCESS);
    }
    let baseline = read_baseline(&dir)?;
    let out_dir = std::env::temp_dir().join(format!("ember-bench-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).map_err(|error| error.to_string())?;
    let result = compare_all(&pairs, &baseline, &out_dir, core);
    let _ = std::fs::remove_dir_all(&out_dir);
    result
}

/// `[TST-29]` — `tests/perf/baseline`: the benchmarks known to fail, one `name: reason` a line,
/// or `name [compiler]: reason` for a failure with one C compiler only (`msvc`, `clang`, `gcc`).
fn read_baseline(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let Ok(text) = std::fs::read_to_string(dir.join("baseline")) else { return Ok(Vec::new()) };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split_once(':')
                .map(|(name, reason)| (name.trim().to_string(), reason.trim().to_string()))
                .ok_or_else(|| format!("`{line}` in {}: a baseline line is `name: reason`", dir.join("baseline").display()))
        })
        .collect()
}

/// `[TST-29]` — a gate whose baseline covers more than a tenth of its cases is `baselined`, not
/// green.
pub(crate) fn gate_state(cases: usize, baselined: usize) -> &'static str {
    if baselined * 10 > cases { "baselined" } else { "green" }
}

fn compare_all(pairs: &[Pair], baseline: &[(String, String)], out_dir: &Path, core: usize) -> Result<ExitCode, String> {
    let requested = std::env::var(ember_branding::cc_var()).ok().filter(|cc| !cc.is_empty());
    let toolchain = ember_build::Toolchain::detect(requested.as_deref()).map_err(|error| error.to_string())?;
    if on_battery() {
        println!("void: the machine is on battery power, unfit for gating [BEN-3]");
        return Ok(ExitCode::SUCCESS);
    }
    pin_to_core(core)?;
    println!("{} benchmark(s), {} on core {core}, {WARMUP} untimed and {TIMED} timed runs each [BEN-1]", pairs.len(), toolchain.name());
    let mut failed = 0;
    let (mut passed, mut voided, mut inconclusive, mut baselined) = (0, 0, 0, 0);
    for pair in pairs {
        let ember = build_ember(&pair.ember, out_dir)?;
        let twin = build_twin(&toolchain, &pair.twin, out_dir, &pair.name)?;
        // `[BEN-3]` — the second copy of the C program, a separate file.
        let copy = out_dir.join(format!("{}_c2{}", pair.name, std::env::consts::EXE_SUFFIX));
        std::fs::copy(&twin, &copy).map_err(|error| error.to_string())?;
        // `[TST-28]` — a checker of identical output.
        let (ember_out, _) = run_once(&ember, pair.stdin.as_deref())?;
        let (c_out, _) = run_once(&twin, pair.stdin.as_deref())?;
        if ember_out != c_out {
            println!("{}: FAIL — its output differs from the C program's", pair.name);
            failed += 1;
            continue;
        }
        let mut outcome = Verdict::Inconclusive;
        let mut shown = String::new();
        for _ in 0..ATTEMPTS {
            let (e, c, c2) = time_pair(&ember, &twin, &copy, pair.stdin.as_deref())?;
            let (e, c, c2) = (side(&e), side(&c), side(&c2));
            let to_c = ratio(e, c);
            outcome = verdict(to_c, ratio(c2, c), pair.threshold);
            shown = format!(
                "Ember {:.2} ms [{:.2}, {:.2}], C {:.2} ms [{:.2}, {:.2}], ratio {:.3} [{:.3}, {:.3}], threshold {:.2}x ({:?})",
                e.median, e.low, e.high, c.median, c.low, c.high, to_c.median, to_c.low, to_c.high, pair.threshold, pair.category
            );
            if outcome != Verdict::Inconclusive {
                break;
            }
        }
        let scoped = format!("{} [{}]", pair.name, toolchain.name());
        let known = baseline.iter().find(|(name, _)| *name == pair.name || *name == scoped).map(|(_, reason)| reason.as_str());
        match (&outcome, known) {
            (Verdict::Fail(_), Some(_)) => baselined += 1,
            (Verdict::Pass, Some(_)) | (Verdict::Fail(_), None) => failed += 1,
            (Verdict::Pass, None) => passed += 1,
            (Verdict::Inconclusive, _) => inconclusive += 1,
            (Verdict::Void(_), _) => voided += 1,
        }
        let word = match (&outcome, known) {
            (Verdict::Fail(why), Some(reason)) => format!("baselined — {reason}; {why} [TST-29]"),
            (Verdict::Pass, Some(_)) => "FAIL — it passes now: take it out of the baseline [TST-29]".to_string(),
            (Verdict::Pass, None) => "pass".to_string(),
            (Verdict::Fail(why), None) => format!("FAIL — {why}"),
            (Verdict::Inconclusive, _) => format!("inconclusive after {ATTEMPTS} runs [BEN-2]"),
            (Verdict::Void(why), _) => format!("void — {why}"),
        };
        println!("{}: {word}\n  {shown}", pair.name);
        // A CI log is read only when signed in; an annotation by anyone.
        if word.starts_with("FAIL") && std::env::var("GITHUB_ACTIONS").is_ok_and(|value| value == "true") {
            println!("::error title=performance gate::{} with {}: {word}; {shown}", pair.name, toolchain.name());
        }
    }
    println!(
        "{passed} passed, {failed} failed, {inconclusive} inconclusive, {voided} void; baseline {baselined} of {} ({}) [TST-29]",
        pairs.len(),
        gate_state(pairs.len(), baselined)
    );
    if failed > 0 {
        eprintln!("{failed} benchmark(s) failed the performance gate [TST-28]");
        return Ok(ExitCode::FAILURE);
    }
    Ok(ExitCode::SUCCESS)
}

/// `[BEN-1]` — the untimed and timed runs of the three programs, interleaved and rotated.
fn time_pair(ember: &Path, twin: &Path, copy: &Path, stdin: Option<&str>) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), String> {
    let programs = [ember, twin, copy];
    for _ in 0..WARMUP {
        for program in programs {
            run_once(program, stdin)?;
        }
    }
    let mut times = [Vec::with_capacity(TIMED), Vec::with_capacity(TIMED), Vec::with_capacity(TIMED)];
    for rep in 0..TIMED {
        for offset in 0..3 {
            let which = (rep + offset) % 3;
            times[which].push(run_once(programs[which], stdin)?.1);
        }
    }
    let [e, c, c2] = times;
    Ok((e, c, c2))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `[BEN-1]` — the distribution-free 95 % interval of the median: for 30 runs, the 10th and
    /// 21st smallest, as the binomial tables give.
    #[test]
    fn the_median_interval_of_thirty_runs_is_the_tenth_to_the_twenty_first() {
        assert_eq!(median_ranks(30), (10, 21));
        assert_eq!(median_ranks(10), (2, 9));
        let samples: Vec<f64> = (1..=30).map(f64::from).collect();
        let s = side(&samples);
        assert_eq!((s.median, s.low, s.high), (15.5, 10.0, 21.0));
    }

    /// `[BEN-1]` — the ratio's interval holds every ratio the two sides' intervals allow.
    #[test]
    fn a_ratio_interval_spans_the_two_intervals() {
        let r = ratio(Side { median: 11.0, low: 10.0, high: 12.0 }, Side { median: 10.0, low: 9.0, high: 11.0 });
        assert!((r.median - 1.1).abs() < 1e-12);
        assert!((r.low - 10.0 / 11.0).abs() < 1e-12 && (r.high - 12.0 / 9.0).abs() < 1e-12);
    }

    fn fit() -> Side {
        Side { median: 1.0, low: 0.99, high: 1.01 }
    }

    /// `[BEN-2]` — fail only when the interval's lower bound is above the threshold; a point above
    /// it whose interval includes it is inconclusive.
    #[test]
    fn a_gate_fails_only_on_the_lower_bound() {
        let above = Side { median: 1.10, low: 1.07, high: 1.13 };
        assert!(matches!(verdict(above, fit(), 1.05), Verdict::Fail(_)));
        let straddles = Side { median: 1.06, low: 1.02, high: 1.10 };
        assert_eq!(verdict(straddles, fit(), 1.05), Verdict::Inconclusive);
        let below = Side { median: 1.01, low: 0.98, high: 1.07 };
        assert_eq!(verdict(below, fit(), 1.05), Verdict::Pass);
    }

    /// `[BEN-3]` — a C program unlike its own copy voids the run.
    #[test]
    fn a_machine_whose_c_program_differs_from_itself_voids_the_run() {
        let ok = Side { median: 1.0, low: 0.97, high: 1.03 };
        let unfit = Side { median: 1.06, low: 1.03, high: 1.09 };
        assert!(matches!(verdict(ok, unfit, 1.05), Verdict::Void(_)));
        assert_eq!(verdict(ok, Side { median: 1.0, low: 0.96, high: 0.985 }, 1.05), Verdict::Pass);
    }

    /// `[TST-29]` — more than a tenth of the cases in the baseline makes the gate `baselined`.
    #[test]
    fn a_large_baseline_makes_the_gate_baselined() {
        assert_eq!(gate_state(20, 2), "green");
        assert_eq!(gate_state(20, 3), "baselined");
        assert_eq!(gate_state(3, 0), "green");
    }

    /// `[BEN-6]` — each category's threshold.
    #[test]
    fn each_category_has_its_threshold() {
        assert_eq!(Category::parse("loop").map(Category::threshold), Some(1.05));
        assert_eq!(Category::parse("simd").map(Category::threshold), Some(1.10));
        assert_eq!(Category::parse("counted").map(Category::threshold), Some(1.15));
        assert_eq!(Category::parse("fast"), None);
    }
}
