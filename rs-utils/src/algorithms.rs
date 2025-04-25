use std::cmp::{max, min};

/// Smallest Range Covering Elements from K Lists
pub fn smallest_range_covering_elements_from_k_lists(
    arrays: &[&[usize]],
) -> (usize, usize, Vec<usize>) {
    let k = arrays.len();
    if k == 0 {
        panic!("Array must not be empty");
    }

    let mut indices = vec![0; k];
    let mut best_range = (0, usize::MAX);
    let mut best_vals = vec![0; k];

    loop {
        let mut current_min = usize::MAX;
        let mut current_max = 0;

        // Find current min and max values among the chosen elements.
        for (i, vec) in arrays.iter().enumerate() {
            if indices[i] >= vec.len() {
                return (best_range.0, best_range.1, best_vals);
            }
            let value = vec[indices[i]];
            current_min = min(current_min, value);
            current_max = max(current_max, value);
        }

        // Update best range if current spread is smaller.
        if current_max - current_min < best_range.1 - best_range.0 {
            best_range = (current_min, current_max);
            for i in 0..k {
                best_vals[i] = arrays[i][indices[i]];
            }
        }

        // Move pointer of the list with the smallest element.
        let mut min_idx = 0;
        for i in 0..k {
            if arrays[i][indices[i]] == current_min {
                min_idx = i;
                break;
            }
        }
        indices[min_idx] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::smallest_range_covering_elements_from_k_lists;

    #[test]
    fn test_smallest_range_covering_elements_from_k_lists() {
        {
            let arrays: Vec<&[usize]> = vec![&[1, 4, 9], &[5, 10], &[11]];
            let result = smallest_range_covering_elements_from_k_lists(arrays.as_slice());
            assert_eq!(result, (9, 11, vec![9, 10, 11]));
        }

        {
            let arrays: Vec<&[usize]> =
                vec![&[4, 10, 15, 24, 26], &[0, 9, 12, 20], &[5, 18, 22, 30]];
            let result = smallest_range_covering_elements_from_k_lists(arrays.as_slice());
            assert_eq!(result, (20, 24, vec![24, 20, 22]));
        }

        {
            let arrays: Vec<&[usize]> = vec![&[1, 2, 3], &[1, 2, 3], &[1, 2, 3]];
            let result = smallest_range_covering_elements_from_k_lists(arrays.as_slice());
            assert_eq!(result, (1, 1, vec![1, 1, 1]));
        }
    }
}
