use lawsynth_core::Seed;

use crate::StatsError;

/// Draws unique population indices with deterministic Fisher-Yates sampling.
pub fn sample_without_replacement(
    population: usize,
    count: usize,
    seed: Seed,
) -> Result<Vec<usize>, StatsError> {
    if count > population {
        return Err(StatsError::SampleExceedsPopulation);
    }
    let mut values = (0..population).collect::<Vec<_>>();
    let mut rng = seed.rng();
    for index in 0..count {
        let offset = (rng.next_u64() as usize) % (population - index);
        values.swap(index, index + offset);
    }
    values.truncate(count);
    Ok(values)
}
