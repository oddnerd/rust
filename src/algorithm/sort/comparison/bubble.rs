//! Implementations of [Bubble Sort](https://en.wikipedia.org/wiki/Bubble_sort).

/// Sort `elements` using naive bubble sort.
///
/// Iteratively 'bubble up' the largest yet to be sorted element by iterating
/// adjacent pairs from the start of the list and swapping the largest element
/// to the end until in sorted position.
///
/// # Performance
/// This method takes O(N<sup>2</sup>) time and consumes O(1) memory.
///
/// # Examples
/// ```
/// use rust::algorithm::sort::comparison::bubble::naive;
///
/// let mut elements = [0, 5, 2, 3, 1, 4];
///
/// naive(&mut elements);
///
/// assert_eq!(elements, [0, 1, 2, 3, 4, 5]);
/// ```
pub fn naive<T: Ord>(elements: &mut [T]) {
    for _sorted in 0..elements.len() {
        let Some(second_last) = elements.len().checked_sub(1) else {
            unreachable!("outer loop would not iterate if length is zero");
        };

        for current_index in 0..second_last {
            let Some(next_index) = current_index.checked_add(1) else {
                unreachable!("loops ensure `current index <= usize::MAX - 1`");
            };

            let (Some(current_element), Some(next_element)) =
                (elements.get(current_index), elements.get(next_index))
            else {
                unreachable!("loops ensure both indexes are within bounds");
            };

            if current_element > next_element {
                elements.swap(current_index, next_index);
            }
        }
    }
}

/// Sort `elements` using optimized bubble sort.
///
/// Fundamentally the same as the [`naive`] implementation, but does not
/// iterate through already sorted elements as well as exiting early when
/// an iteration proves the remaining elements are already sorted.
///
/// # Performance
/// This method takes O(N<sup>2</sup>) time and consumes O(1) memory.
///
/// # Examples
/// ```
/// use rust::algorithm::sort::comparison::bubble::optimized;
///
/// let mut elements = [0, 5, 2, 3, 1, 4];
///
/// optimized(&mut elements);
///
/// assert_eq!(elements, [0, 1, 2, 3, 4, 5]);
/// ```
pub fn optimized<T: Ord>(elements: &mut [T]) {
    for sorted_position in (0..elements.len()).rev() {
        let mut sorted = true;

        // `elements[sorted_position..]` are already sorted.
        for current_index in 0..sorted_position {
            let Some(next_index) = current_index.checked_add(1) else {
                unreachable!("inner loop prevents ensures within bounds");
            };

            let (Some(current_element), Some(next_element)) =
            (elements.get(current_index), elements.get(next_index))
            else {
                unreachable!("loops ensure both indexes are within bounds");
            };

            if current_element > next_element {
                elements.swap(current_index, next_index);
                sorted = false;
            }
        }

        // If no elements were swapped, then the remaining are already sorted.
        if sorted {
            break;
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::assertions_on_result_states,
    clippy::indexing_slicing
)]
mod test {
    use super::*;

    mod naive {
        use super::*;

        #[test]
        fn empty() {
            let mut elements = [usize::default(); 0];

            naive(&mut elements);

            assert_eq!(elements, []);
        }

        #[test]
        fn single_element() {
            let mut elements = [0];

            naive(&mut elements);

            assert_eq!(elements, [0]);
        }

        #[test]
        fn already_sorted() {
            let mut elements = [0, 1, 2, 3, 4, 5];

            naive(&mut elements);

            assert_eq!(elements, [0, 1, 2, 3, 4, 5]);
        }

        #[test]
        fn must_swap() {
            let mut elements = [1, 0];

            naive(&mut elements);

            assert_eq!(elements, [0, 1]);
        }

        #[test]
        fn odd_length() {
            let mut elements = [2, 1, 0];

            naive(&mut elements);

            assert_eq!(elements, [0, 1, 2]);
        }

        #[test]
        fn multiple_swaps() {
            let mut elements = [2, 0, 3, 1];

            naive(&mut elements);

            assert_eq!(elements, [0, 1, 2, 3]);
        }
    }

    mod optimized {
        use super::*;

        #[test]
        fn empty() {
            let mut elements = [usize::default(); 0];

            optimized(&mut elements);

            assert_eq!(elements, []);
        }

        #[test]
        fn single_element() {
            let mut elements = [0];

            optimized(&mut elements);

            assert_eq!(elements, [0]);
        }

        #[test]
        fn already_sorted() {
            let mut elements = [0, 1, 2, 3, 4, 5];

            optimized(&mut elements);

            assert_eq!(elements, [0, 1, 2, 3, 4, 5]);
        }

        #[test]
        fn must_swap() {
            let mut elements = [1, 0];

            optimized(&mut elements);

            assert_eq!(elements, [0, 1]);
        }

        #[test]
        fn odd_length() {
            let mut elements = [2, 1, 0];

            optimized(&mut elements);

            assert_eq!(elements, [0, 1, 2]);
        }

        #[test]
        fn multiple_swaps() {
            let mut elements = [2, 0, 3, 1];

            optimized(&mut elements);

            assert_eq!(elements, [0, 1, 2, 3]);
        }
    }
}