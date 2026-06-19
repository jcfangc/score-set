mod features;
mod finite_enum;
mod fixed_tuple;

use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: xtask gen --max <N>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "gen" => {
            let max = args
                .get(3)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(8);

            // 1. Generate all files first (so the module tree is complete).
            features::generate(max);
            fixed_tuple::generate(max);
            finite_enum::generate(max);

            // 2. Format all generated files in one pass.
            let out_dir = std::env::current_dir().unwrap();
            format_file(&out_dir.join("src").join("fixed_tuple.rs"));
            format_file(&out_dir.join("src").join("finite_enum.rs"));
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            std::process::exit(1);
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
