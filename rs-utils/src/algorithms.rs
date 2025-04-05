use std::cmp::{max, min};

/// Smallest Range Covering Elements from K Lists
pub fn smallest_range_covering_elements_from_k_lists(
    arrays: &[Vec<usize>],
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

pub fn scale_f64_to_u128(val: f64) -> Option<u128> {
    if !val.is_finite() || val.is_sign_negative() {
        return None;
    }

    // Scale by a large factor to preserve precision
    let scaled = val * (1u128 << 64) as f64;
    Some(scaled as u128)
}

#[cfg(test)]
mod tests {
    use super::smallest_range_covering_elements_from_k_lists;

    #[test]
    fn test_smallest_range_covering_elements_from_k_lists() {
        {
            let arrays = vec![vec![1, 4, 9], vec![5, 10], vec![11]];
            let result = smallest_range_covering_elements_from_k_lists(arrays.as_slice());
            assert_eq!(result, (9, 11, vec![9, 10, 11]));
        }

        {
            let arrays = vec![
                vec![4, 10, 15, 24, 26],
                vec![0, 9, 12, 20],
                vec![5, 18, 22, 30],
            ];
            let result = smallest_range_covering_elements_from_k_lists(arrays.as_slice());
            assert_eq!(result, (20, 24, vec![24, 20, 22]));
        }

        {
            let arrays = vec![vec![1, 2, 3], vec![1, 2, 3], vec![1, 2, 3]];
            let result = smallest_range_covering_elements_from_k_lists(arrays.as_slice());
            assert_eq!(result, (1, 1, vec![1, 1, 1]));
        }
    }
}
