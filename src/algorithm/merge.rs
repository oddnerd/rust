//! Combine (merge) sorted collections whilst preserving order.

/// An [`Iterator`] to traverse two other sorted [`Iterator`] in sorted order.
///
/// # Examples
/// ```
/// use rust::algorithm::merge::Iter;
///
/// let instance = Iter::new([0, 2, 4].into_iter(), [1, 3, 5].into_iter());
///
/// assert!(instance.eq([0, 1, 2, 3, 4, 5]));
/// ```
#[derive(Debug)]
pub struct Iter<T: Ord, I: Iterator<Item = T>> {
    /// The first [`Iterator`] to merge.
    first: core::iter::Peekable<I>,

    /// The second [`Iterator`] to merge.
    second: core::iter::Peekable<I>,
}

impl<T: Ord, I: Iterator<Item = T>> Iter<T, I> {
    /// Construct an [`Iter`] from two other [`Iterator`].
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    pub fn new(first: I, second: I) -> Self {
        Iter {
            first: first.peekable(),
            second: second.peekable(),
        }
    }
}

impl<T: Ord, I: Iterator<Item = T>> Iterator for Iter<T, I> {
    type Item = T;

    /// Obtain the next item in sorted order.
    ///
    /// # Performance
    /// This method takes O(1) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// use rust::algorithm::merge::Iter;
    ///
    /// let mut instance = Iter::new(
    ///     [0, 2, 4].into_iter(),
    ///     [1, 3, 5].into_iter()
    /// );
    ///
    /// assert_eq!(instance.next(), Some(0));
    /// assert_eq!(instance.next(), Some(1));
    /// assert_eq!(instance.next(), Some(2));
    /// assert_eq!(instance.next(), Some(3));
    /// assert_eq!(instance.next(), Some(4));
    /// assert_eq!(instance.next(), Some(5));
    /// assert_eq!(instance.next(), None);
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(left) = self.first.peek() {
            if let Some(right) = self.second.peek() {
                if left <= right {
                    self.first.next()
                } else {
                    self.second.next()
                }
            } else {
                self.first.next()
            }
        } else {
            self.second.next()
        }
    }
}

/// Merge two slices into one output slice.
///
/// For the convenience of implementation to not depend on a particular
/// executor, this method executes synchronously within the singly calling
/// thread. However, the implementation is of a parallel algorithm that could
/// be trivially modified to execute asynchronously.
///
/// # Performance
/// Synchronous: This method takes O(N * log N) time and consumes O(N) memory.
/// Asynchronous: This method takes O(log^2 N) time and consumes O(N) memory.
///
/// # Examples
/// ```
/// use rust::algorithm::merge::parallel;
///
/// let first  = [0, 2, 4];
/// let second = [1, 3, 5];
/// let mut output = [0; 6];
///
/// parallel(&first, &second, &mut output);
///
/// assert_eq!(output, [0, 1, 2, 3, 4, 5]);
/// ```
#[allow(clippy::indexing_slicing)]
#[allow(clippy::arithmetic_side_effects)]
pub fn parallel<T: Ord + Clone>(first: &[T], second: &[T], output: &mut [T]) {
    if first.len() < second.len() {
        return parallel(second, first, output);
    }

    if first.is_empty() {
        return;
    }

    let middle = first.len() / 2;

    // NOTE: binary search is O(log N).
    let intersect = match second.binary_search(&first[middle]) {
        Err(index) | Ok(index) => index,
    };

    let (first_left, first_right) = first.split_at(middle);
    let (second_left, second_right) = second.split_at(intersect);
    let (output_left, output_right) = output.split_at_mut(middle + intersect);

    output_right[0] = first_right[0].clone();
    let output_right = &mut output_right[1..];
    let first_right = &first_right[1..];

    // The following calls could be executed concurrently.
    parallel(first_left, second_left, output_left);
    parallel(first_right, second_right, output_right);
}

/// Merge two halves of a slice in-place.
///
/// Naive implementation, n<sup>2</sup> time complexity.
///
/// # Examples
/// ```
/// use rust::algorithm::merge::inplace;
/// let mut slice = [0,2,4,1,3,5];
/// inplace(&mut slice, 3);
/// assert_eq!(slice, [0,1,2,3,4,5]);
/// ```
pub fn inplace<T>(slice: &mut [T], mut middle: usize)
where
    T: Ord,
{
    let mut left = 0;
    let mut right = middle;

    while (left < middle) && (right < slice.len()) {
        if slice[left] < slice[right] {
            left += 1;
        } else {
            slice[left..=right].rotate_right(1);
            left += 1;
            middle += 1;
            right += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod iter {
        use super::*;

        #[test]
        fn first_empty() {
            let first = [];
            let second = [0];
            let result: Vec<&i32> = Iter::new(first.iter(), second.iter()).collect();

            assert_eq!(result.len(), 1);
            assert_eq!(*result[0], 0);
        }

        #[test]
        fn second_empty() {
            let first = [0];
            let second = [];
            let result: Vec<&i32> = Iter::new(first.iter(), second.iter()).collect();

            assert_eq!(result.len(), 1);
            assert_eq!(*result[0], 0);
        }

        #[test]
        fn first_greater() {
            let first = [1];
            let second = [0];
            let result: Vec<&i32> = Iter::new(first.iter(), second.iter()).collect();

            assert_eq!(result.len(), 2);
            assert_eq!(*result[0], 0);
            assert_eq!(*result[1], 1);
        }

        #[test]
        fn second_greater() {
            let first = [0];
            let second = [1];
            let result: Vec<&i32> = Iter::new(first.iter(), second.iter()).collect();

            assert_eq!(result.len(), 2);
            assert_eq!(*result[0], 0);
            assert_eq!(*result[1], 1);
        }

        #[test]
        fn back_and_forth() {
            let first = [1, 2];
            let second = [0, 3];
            let result: Vec<&i32> = Iter::new(first.iter(), second.iter()).collect();

            assert_eq!(result.len(), 4);
            assert_eq!(*result[0], 0);
            assert_eq!(*result[1], 1);
            assert_eq!(*result[2], 2);
            assert_eq!(*result[3], 3);
        }
    }

    mod parallel {
        use super::*;

        #[test]
        fn first_empty() {
            let first = [];
            let second = [0];
            let mut output = vec![0; 1];
            parallel(&first, &second, &mut output);

            assert_eq!(output.len(), 1);
            assert_eq!(output[0], 0);
        }

        #[test]
        fn second_empty() {
            let first = [0];
            let second = [];
            let mut output = vec![0; 1];
            parallel(&first, &second, &mut output);

            assert_eq!(output.len(), 1);
            assert_eq!(output[0], 0);
        }

        #[test]
        fn first_greater() {
            let first = [1];
            let second = [0];
            let mut output = vec![0; 2];
            parallel(&first, &second, &mut output);

            assert_eq!(output.len(), 2);
            assert_eq!(output[0], 0);
            assert_eq!(output[1], 1);
        }

        #[test]
        fn second_greater() {
            let first = [0];
            let second = [1];
            let mut output = vec![0; 2];
            parallel(&first, &second, &mut output);

            assert_eq!(output.len(), 2);
            assert_eq!(output[0], 0);
            assert_eq!(output[1], 1);
        }

        #[test]
        fn back_and_forth() {
            let first = [1, 2];
            let second = [0, 3];
            let mut output = vec![0; 4];
            parallel(&first, &second, &mut output);

            assert_eq!(output.len(), 4);
            assert_eq!(output[0], 0);
            assert_eq!(output[1], 1);
            assert_eq!(output[2], 2);
            assert_eq!(output[3], 3);
        }
    }

    mod inplace {
        use super::*;

        #[test]
        fn first_empty() {
            let mut slice = [0];
            inplace(&mut slice, 0);
            assert_eq!(slice, [0]);
        }

        #[test]
        fn second_empty() {
            let mut slice = [0];
            inplace(&mut slice, 1);
            assert_eq!(slice, [0]);
        }

        #[test]
        fn first_greater() {
            let mut slice = [1, 0];
            inplace(&mut slice, 1);
            assert_eq!(slice, [0, 1]);
        }

        #[test]
        fn second_greater() {
            let mut slice = [0, 1];
            inplace(&mut slice, 1);
            assert_eq!(slice, [0, 1]);
        }

        #[test]
        fn back_and_forth() {
            let mut slice = [0, 3, 1, 2];
            inplace(&mut slice, 2);
            assert_eq!(slice, [0, 1, 2, 3]);
        }
    }
}
