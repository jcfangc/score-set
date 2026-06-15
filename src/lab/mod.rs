/// Test helper: compute GC ratio in [0, 1].
pub fn gc_ratio(dna: &str) -> f64 {
    if dna.is_empty() {
        return 0.0;
    }
    let gc_count = dna.chars().filter(|c| *c == 'G' || *c == 'C').count();
    gc_count as f64 / dna.len() as f64
}

#[cfg(test)]
mod ergonomics;
