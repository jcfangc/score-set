use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: xtask gen");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "gen" => {
            generate_f64();
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

/// Generate `src/metric_f64.rs` from `src/metric_f32.rs` by replacing
/// `f32` → `f64` for the Score type alias, then `32` → `64` for all
/// type/function name suffixes. Companion test files are generated the same way.
fn generate_f64() {
    let out_dir = std::env::current_dir().unwrap();

    // --- Main source file ---
    let src_f32 = out_dir.join("src").join("metric_f32.rs");
    let src_f64 = out_dir.join("src").join("metric_f64.rs");

    let content = std::fs::read_to_string(&src_f32).expect("failed to read src/metric_f32.rs");
    let content = content.replace("pub type Score32 = f32;", "pub type Score64 = f64;");
    let content = content.replace("Score32 = f32", "Score64 = f64");
    let content = content.replace("32", "64");

    std::fs::write(&src_f64, &content).expect("failed to write src/metric_f64.rs");
    format_file(&src_f64);

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
            let dest = test_f64_dir.join(name_str);
            std::fs::write(&dest, &test_content)
                .unwrap_or_else(|e| eprintln!("warning: {dest:?}: {e}"));
            format_file(&dest);
        }
    }
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
