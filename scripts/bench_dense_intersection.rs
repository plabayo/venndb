//! Standalone kernel benchmark. Compile with rustc --edition=2024 -C opt-level=3 and run the executable.
use std::hint::black_box;
use std::time::Instant;

const BLOCK_WORDS: usize = 128;
const SAMPLES: usize = 15;
type Kernel = fn(&[Vec<u64>]) -> (Vec<u64>, usize);

fn intersect(mask: &mut [u64], input: &[u64]) {
    for (word, other) in mask.iter_mut().zip(input) {
        *word &= *other;
    }
}

fn finish(mask: Vec<u64>) -> (Vec<u64>, usize) {
    let count = mask.iter().map(|w| w.count_ones() as usize).sum();
    (mask, count)
}

// Dense Is filters of equal span, matching the operations in Plan::combine.
fn passes(inputs: &[Vec<u64>]) -> (Vec<u64>, usize) {
    let mut mask = inputs[0].clone();
    for input in &inputs[1..] {
        intersect(&mut mask, input);
    }
    finish(mask)
}

fn blocks<const SKIP: bool>(inputs: &[Vec<u64>]) -> (Vec<u64>, usize) {
    let mut mask = inputs[0].clone();
    for (index, block) in mask.chunks_mut(BLOCK_WORDS).enumerate() {
        let start = index * BLOCK_WORDS;
        let end = start + block.len();
        for (i, input) in inputs[1..].iter().enumerate() {
            intersect(block, &input[start..end]);
            if SKIP && i + 2 < inputs.len() && block.iter().all(|w| *w == 0) {
                break;
            }
        }
    }
    finish(mask)
}

fn inputs(words: usize, filters: usize, clustered: bool) -> Vec<Vec<u64>> {
    let mut state = 9357u64;
    (0..filters)
        .map(|column| {
            (0..words)
                .map(|word| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    // Two ~50%-dense indexes with disjoint bits in 7 of 8 regions.
                    // Regions are 8 KiB per index, independent of kernel block size.
                    if clustered && column < 2 && (word / 1024) % 8 != 0 {
                        if column == 0 {
                            0x5555555555555555
                        } else {
                            0xaaaaaaaaaaaaaaaa
                        }
                    } else {
                        state
                    }
                })
                .collect()
        })
        .collect()
}

fn main() {
    let kernels: [Kernel; 3] = [passes, blocks::<false>, blocks::<true>];
    println!("rows filters layout repeats passes_us blocks_us skip_us");
    for words in [16384, 2097152] {
        for filters in [2, 4, 8] {
            for clustered in [false, true] {
                let inputs = inputs(words, filters, clustered);
                let expected = passes(&inputs);
                for kernel in kernels {
                    assert_eq!(kernel(&inputs), expected);
                }
                drop(expected);
                let repeats = if words == 16384 { 80 } else { 2 };
                let mut timings = [Vec::new(), Vec::new(), Vec::new()];
                for sample in 0..SAMPLES {
                    for offset in 0..3 {
                        let k = (sample + offset) % 3;
                        let start = Instant::now();
                        for _ in 0..repeats {
                            black_box(kernels[k](black_box(&inputs)));
                        }
                        timings[k].push(start.elapsed().as_secs_f64() * 1e6 / repeats as f64);
                    }
                }
                let medians = timings.map(|mut times| {
                    times.sort_by(f64::total_cmp);
                    times[SAMPLES / 2]
                });
                let layout = if clustered {
                    "clustered"
                } else {
                    "independent"
                };
                println!(
                    "{} {filters} {layout} {repeats} {:.2} {:.2} {:.2}",
                    words * 64,
                    medians[0],
                    medians[1],
                    medians[2]
                );
            }
        }
    }
}
