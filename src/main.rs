use duotrigordle::{init_word_list, Duotrigordle};

use rayon::prelude::*;

fn main() {
    let words = init_word_list();

    const CHECK_COUNT: usize = 1;
    const THREAD_COUNT: usize = 4;

    rayon::ThreadPoolBuilder::new()
        .num_threads(THREAD_COUNT)
        .build_global()
        .unwrap();

    let v_words = words.iter().map(|e| *e).collect::<Vec<_>>();

    let out = words
        .par_iter()
        .map(|w| {
            let mut solve = 0;
            let mut total = 0;
            for _ in 0..CHECK_COUNT {
                let mut dt = Duotrigordle::new_single_fixed(w, v_words.clone());
                if dt.solveable_from(0).is_some() {
                    solve += 1;
                }
                total += 1;
            }
            (w, solve, total)
        })
        .map(|(w, s, t)| (w, binomial_ci(s, t, 2.0)))
        .collect::<Vec<_>>();

    for res in out {
        println!(
            "Guess {} p: {:.2}% 2σ[{:.2}% - {:.2}%]",
            res.0.iter().collect::<String>(),
            res.1 .1 * 100.0,
            res.1 .0 * 100.0,
            res.1 .2 * 100.0
        );
    }
}

fn binomial_ci(success: usize, total: usize, stddev: f64) -> (f64, f64, f64) {
    let p_est = (success as f64) / (total as f64);
    let dev = stddev * (p_est * (1.0 - p_est) / total as f64).sqrt();
    (p_est - dev, p_est, p_est + dev)
}
