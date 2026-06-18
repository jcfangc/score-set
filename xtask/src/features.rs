use std::fs;

/// Update `Cargo.toml` feature gates section (marker-based replacement).
///
/// Reads `Cargo.toml`, finds the `# >>>> BEGIN generated features` /
/// `# <<<< END generated features` markers, and replaces the content
/// between them with generated `fixed-tuple-N`, `finite-enum-N`, and
/// umbrella `level-N` feature definitions.
pub fn generate(max: usize) {
    let cargo_toml_path = std::env::current_dir().unwrap().join("Cargo.toml");

    assert!(max >= 1, "max must be >= 1");

    let original = fs::read_to_string(&cargo_toml_path).unwrap_or_else(|e| {
        eprintln!("error reading {}: {e}", cargo_toml_path.display());
        std::process::exit(1);
    });

    let begin_marker = "# >>>> BEGIN generated features\n";
    let end_marker = "# <<<< END generated features\n";

    let begin_pos = original.find(begin_marker).unwrap_or_else(|| {
        eprintln!(
            "error: {} missing begin marker: {:?}",
            cargo_toml_path.display(),
            begin_marker.trim()
        );
        std::process::exit(1);
    });
    // end marker must appear after begin marker
    let end_pos = original[begin_pos..].find(end_marker).unwrap_or_else(|| {
        eprintln!(
            "error: {} missing end marker after begin marker",
            cargo_toml_path.display()
        );
        std::process::exit(1);
    }) + begin_pos;

    let before = &original[..begin_pos + begin_marker.len()];
    let after = &original[end_pos..];

    let generated = build_features_section(max);

    let new_content = format!("{before}{generated}{after}");
    fs::write(&cargo_toml_path, &new_content).unwrap_or_else(|e| {
        eprintln!("error writing {}: {e}", cargo_toml_path.display());
        std::process::exit(1);
    });

    println!(
        "Updated {} features: fixed-tuple 1–{max}, finite-enum 1–{max}",
        cargo_toml_path.display()
    );
}

/// Build the generated `[features]` block content.
fn build_features_section(max: usize) -> String {
    let default_level = default_level_for(max);
    let mut s = String::new();

    // [features] header
    s.push_str("[features]\n");
    s.push_str(&format!("default = [\"{default_level}\"]\n\n"));

    // ---- fixed-tuple chain ----
    s.push_str("# Layer 1 — fixed tuples\n");
    fixed_chain(&mut s, "fixed-tuple", max);

    // ---- finite-enum chain ----
    s.push_str("# Layer 2 — finite enums\n");
    fixed_chain(&mut s, "finite-enum", max);

    // ---- per-layer umbrellas ----
    s.push_str("# Per-layer umbrellas\n");
    for level in umbrella_levels(max) {
        s.push_str(&format!(
            "fixed-level-{level}   = [\"fixed-tuple-{level}\"]\n"
        ));
    }
    s.push('\n');
    for level in umbrella_levels(max) {
        s.push_str(&format!(
            "finite-level-{level}   = [\"finite-enum-{level}\"]\n"
        ));
    }
    s.push('\n');

    // ---- combined umbrellas ----
    s.push_str("# Combined umbrellas\n");
    for level in umbrella_levels(max) {
        s.push_str(&format!(
            "level-{level}   = [\"fixed-level-{level}\", \"finite-level-{level}\"]\n"
        ));
    }
    s.push('\n');

    s
}

/// Generate a hierarchical feature chain like `{prefix}-1 = []` through
/// `{prefix}-{max} = ["{prefix}-{max-1}"]`, with grouping comments.
fn fixed_chain(s: &mut String, prefix: &str, max: usize) {
    struct Group {
        label: &'static str,
        start: usize,
        end: usize,
    }

    let groups = if max > 8 {
        vec![
            Group {
                label: "arities 1–8",
                start: 1,
                end: 8.min(max),
            },
            Group {
                label: "up to 32",
                start: 9,
                end: 32.min(max),
            },
            Group {
                label: "up to 64",
                start: 33,
                end: 64.min(max),
            },
            Group {
                label: "up to 128",
                start: 65,
                end: max,
            },
        ]
    } else {
        vec![Group {
            label: "arities 1–8",
            start: 1,
            end: max,
        }]
    };

    for group in &groups {
        if group.start > group.end {
            continue;
        }
        s.push_str(&format!("# {}\n", group.label));
        for n in group.start..=group.end {
            if n == 1 {
                s.push_str(&format!("{prefix}-1 = []\n"));
            } else {
                let prev = n - 1;
                s.push_str(&format!("{prefix}-{n} = [\"{prefix}-{prev}\"]\n"));
            }
        }
    }
    s.push('\n');
}

/// The power-of-two levels up to `max`.
fn umbrella_levels(max: usize) -> Vec<usize> {
    [8usize, 16, 32, 64, 128]
        .iter()
        .copied()
        .filter(|&l| l <= max)
        .collect()
}

fn default_level_for(max: usize) -> &'static str {
    if max <= 8 {
        "level-8"
    } else if max <= 16 {
        "level-16"
    } else if max <= 32 {
        "level-32"
    } else if max <= 64 {
        "level-64"
    } else {
        "level-128"
    }
}
