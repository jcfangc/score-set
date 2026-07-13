use std::fs;

/// Inject `fixed-tuple-N` and `level-N` feature flags into `Cargo.toml`.
///
/// The flags live between two marker comments:
/// ```toml
/// # >>>> BEGIN generated features
/// ...
/// # <<<< END generated features
/// ```
/// Everything between the markers is replaced.
pub fn generate(max: usize) {
    let out_dir = std::env::current_dir().unwrap();
    let cargo_toml = out_dir.join("Cargo.toml");

    let content = fs::read_to_string(&cargo_toml).unwrap_or_else(|e| {
        eprintln!("error reading Cargo.toml: {e}");
        std::process::exit(1);
    });

    let begin = "# >>>> BEGIN generated features";
    let end = "# <<<< END generated features";

    let begin_pos = content.find(begin).unwrap_or_else(|| {
        eprintln!("error: marker '{begin}' not found in Cargo.toml");
        std::process::exit(1);
    });
    let end_pos = content.find(end).unwrap_or_else(|| {
        eprintln!("error: marker '{end}' not found in Cargo.toml");
        std::process::exit(1);
    });

    let before = &content[..begin_pos + begin.len()];
    let after = &content[end_pos..];

    let default_level = default_level_str();
    let mut features = String::from("\n");

    // Per-arity features with chain inheritance
    features.push_str(&format!(
        "# Per-arity feature gates. Enabling `fixed-tuple-{n}` pulls in 1–{n}.\n",
        n = max
    ));
    features.push_str("fixed-tuple-1 = []\n");
    for n in 2..=max {
        features.push_str(&format!(
            "fixed-tuple-{n} = [\"fixed-tuple-{p}\"]\n",
            p = n - 1
        ));
    }

    features.push_str("\n# Umbrella features at power-of-two boundaries.\n");
    for level in power_of_two_levels(max) {
        features.push_str(&format!("level-{level} = [\"fixed-tuple-{level}\"]\n"));
    }

    // Default is always level-8 (fast compile).  Higher arities opt-in.
    features.push_str("default = [\"level-8\"]\n");

    let merged = format!("{before}{features}\n{after}");
    fs::write(&cargo_toml, &merged).unwrap_or_else(|e| {
        eprintln!("error writing Cargo.toml: {e}");
        std::process::exit(1);
    });

    println!("Updated Cargo.toml features: fixed-tuple-1..{max}, default={default_level}");
}

fn power_of_two_levels(max: usize) -> Vec<usize> {
    let mut levels = vec![];
    let mut n = 8;
    while n <= max {
        levels.push(n);
        n *= 2;
    }
    if levels.last() != Some(&max) && max > 8 && !max.is_power_of_two() {
        // Don't add max if it's not a power of two; levels are only for powers of two
        // The per-arity chain already covers everything
    }
    levels
}

fn default_level_str() -> &'static str {
    "level-8"
}
