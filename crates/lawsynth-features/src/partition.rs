use std::ops::Range;

/// Splits `rows` into at most `threads` contiguous, gap-free ranges.
///
/// The split is deterministic and depends only on `rows` and `threads`, never on
/// scheduling: the number of chunks is `clamp(threads, 1, rows)`, the base chunk
/// size is `rows / chunks`, and the first `rows % chunks` chunks each take one
/// extra row. The returned ranges tile `0..rows` exactly, in ascending order,
/// with no gaps or overlaps. Concatenating per-chunk output in this order
/// therefore reproduces the serial row order exactly.
///
/// * `rows == 0` yields an empty partition (no work).
/// * `threads == 0` or `threads == 1` yields a single `0..rows` chunk.
/// * `threads > rows` is capped so no empty chunk is ever produced.
pub fn row_partitions(rows: usize, threads: usize) -> Vec<Range<usize>> {
    if rows == 0 {
        return Vec::new();
    }
    let chunks = threads.clamp(1, rows);
    let base = rows / chunks;
    let remainder = rows % chunks;
    let mut partitions = Vec::with_capacity(chunks);
    let mut start = 0;
    for index in 0..chunks {
        let extra = usize::from(index < remainder);
        let end = start + base + extra;
        partitions.push(start..end);
        start = end;
    }
    partitions
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every partition must tile `0..rows` exactly: contiguous, gap-free, ordered.
    fn assert_tiles(rows: usize, threads: usize) {
        let partitions = row_partitions(rows, threads);
        let mut cursor = 0;
        for range in &partitions {
            assert_eq!(range.start, cursor, "gap or overlap at {rows}/{threads}");
            assert!(range.end >= range.start, "reversed range at {rows}/{threads}");
            cursor = range.end;
        }
        assert_eq!(cursor, rows, "partitions must cover all rows at {rows}/{threads}");
    }

    #[test]
    fn zero_rows_produce_no_chunks() {
        assert!(row_partitions(0, 4).is_empty());
        assert!(row_partitions(0, 0).is_empty());
    }

    #[test]
    fn zero_or_one_thread_is_a_single_chunk() {
        assert_eq!(row_partitions(10, 0), vec![0..10]);
        assert_eq!(row_partitions(10, 1), vec![0..10]);
    }

    #[test]
    fn even_split_has_equal_chunks() {
        assert_eq!(row_partitions(8, 4), vec![0..2, 2..4, 4..6, 6..8]);
    }

    #[test]
    fn uneven_split_front_loads_the_remainder() {
        // 10 rows over 3 threads: sizes 4,3,3 (first `10 % 3 == 1` chunk gets +1).
        assert_eq!(row_partitions(10, 3), vec![0..4, 4..7, 7..10]);
        // 10 rows over 4 threads: sizes 3,3,2,2 (first `10 % 4 == 2` chunks get +1).
        assert_eq!(row_partitions(10, 4), vec![0..3, 3..6, 6..8, 8..10]);
    }

    #[test]
    fn more_threads_than_rows_never_makes_empty_chunks() {
        let partitions = row_partitions(3, 100);
        assert_eq!(partitions, vec![0..1, 1..2, 2..3]);
        assert!(partitions.iter().all(|range| range.end > range.start));
    }

    #[test]
    fn tiles_exactly_across_many_shapes() {
        for rows in 0..40 {
            for threads in 0..12 {
                assert_tiles(rows, threads);
            }
        }
    }
}
