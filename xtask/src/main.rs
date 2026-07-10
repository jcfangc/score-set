mod features;
mod gen_macros;

use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: xtask gen [--max <N>]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "gen" => {
            let max = parse_max(&args);
            gen_macros::generate(max);
            features::generate(max);
            generate_f64();
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn parse_max(args: &[String]) -> usize {
    for i in 0..args.len() {
        if args[i] == "--max" {
            if let Some(val) = args.get(i + 1) {
                return val.parse().unwrap_or_else(|_| {
                    eprintln!("invalid --max value: {val}");
                    std::process::exit(1);
                });
            }
        }
    }
    64 // default
}

// ---------------------------------------------------------------------------
// f64 generation — string replacement from f32 sources
// ---------------------------------------------------------------------------

/// Generate `src/metric_f64.rs` from `src/metric_f32.rs` and
/// `src/gen_score_set64.rs` from `src/gen_score_set32.rs`.
fn generate_f64() {
    let out_dir = std::env::current_dir().unwrap();

    // --- metric_f64.rs ---
    replace_and_write(&out_dir, "src/metric_f32.rs", "src/metric_f64.rs", |s| {
        s.replace("libm::expf", "libm::exp")
            .replace("libm::logf", "libm::log")
            .replace("f32", "f64")
            .replace("32", "64")
    });

    // --- gen_score_set64.rs ---
    replace_and_write(
        &out_dir,
        "src/gen_score_set32.rs",
        "src/gen_score_set64.rs",
        |s| {
            s.replace("f32", "f64")
                .replace("32", "64")
                .replace("macro_rules! score_set32", "macro_rules! score_set64")
        },
    );

    // --- Test files ---
    let test_f32_dir = out_dir.join("src").join("metric_f32");
    let test_f64_dir = out_dir.join("src").join("metric_f64");
    std::fs::create_dir_all(&test_f64_dir).ok();

    for entry in std::fs::read_dir(&test_f32_dir).expect("failed to read metric_f32 test directory")
    {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name_str = name.to_str().unwrap();
        if name_str.ends_with(".rs") {
            let test_content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let test_content = test_content.replace("f32", "f64");
            let test_content = test_content.replace("32", "64");
            let test_content = test_content.replace("score_set32!", "score_set64!");
            let test_content =
                test_content.replace("use crate::score_set32;", "use crate::score_set64;");
            let dest = test_f64_dir.join(name_str);
            std::fs::write(&dest, &test_content).unwrap_or_else(|e| {
                eprintln!("warning: {dest:?}: {e}");
            });
            format_file(&dest);
        }
    }

    format_file(&out_dir.join("src").join("metric_f64.rs"));
    format_file(&out_dir.join("src").join("gen_score_set64.rs"));
}

fn replace_and_write(
    out_dir: &std::path::Path,
    src_rel: &str,
    dst_rel: &str,
    f: impl Fn(String) -> String,
) {
    let src = out_dir.join(src_rel);
    let dst = out_dir.join(dst_rel);

    let content = std::fs::read_to_string(&src).unwrap_or_else(|e| {
        eprintln!("error reading {}: {e}", src.display());
        std::process::exit(1);
    });
    let content = f(content);
    std::fs::write(&dst, &content).unwrap_or_else(|e| {
        eprintln!("error writing {}: {e}", dst.display());
        std::process::exit(1);
    });
}

/// Run `cargo fmt` on a generated file.
fn format_file(path: &std::path::Path) {
    let status = Command::new("cargo")
        .args(["fmt", "--", path.to_str().unwrap()])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("warning: cargo fmt failed on {}: {e}", path.display());
            std::process::exit(1);
        });
    if !status.success() {
        eprintln!(
            "warning: cargo fmt exited with non-zero status on {}",
            path.display()
        );
    }
}
