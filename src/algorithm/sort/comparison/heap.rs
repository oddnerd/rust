//! Implementations of [Heap Sort](https://en.wikipedia.org/wiki/Heapsort).

#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::indexing_slicing)]

/// Index of the left child of the node at `index` in a binary heap.
fn left_child(index: usize) -> usize {
    2 * index + 1
}

/// Index of the right child of the node at `index` in a binary heap.
fn right_child(index: usize) -> usize {
    2 * index + 2
}

/// Index of the parent of the node at `index` in a binary heap.
fn parent(index: usize) -> usize {
    (index - 1) / 2
}

/// Move a misplaced node down a heap into the correct level.
mod sift_down {
    use super::left_child;
    use super::parent;
    use super::right_child;

    /// Sift the root of a binary `max_heap` down to the correct position.
    ///
    /// Swap the current root with the greatest child until both children are
    /// less than that root, thereby repairing a max-heap with invalid root.
    ///
    /// # Performance
    /// This method takes O(log N) time and consumes O(1) memory.
    pub(super) fn top_down<T: Ord>(max_heap: &mut [T]) {
        let mut root = 0;

        while left_child(root) < max_heap.len() {
            let greatest_child = if right_child(root) < max_heap.len()
                && max_heap[left_child(root)] < max_heap[right_child(root)]
            {
                right_child(root)
            } else {
                left_child(root)
            };

            if max_heap[root] < max_heap[greatest_child] {
                max_heap.swap(root, greatest_child);
                root = greatest_child;
            } else {
                return;
            }
        }
    }

    /// Sift the root of a binary `max_heap` down to the correct position.
    ///
    /// Traverse the heap down to the leaves (this is presumably where the
    /// current root value came from), and then traverse upward a node is found
    /// which is greater than the value being sifted down.
    ///
    /// This uses fewer comparison than [`top_down`], but will likely have
    /// worse performance for cheap comparisons.
    ///
    /// # Performance
    /// This method takes O(log N) time and consumes O(1) memory.
    pub(super) fn bottom_up<T: Ord>(max_heap: &mut [T]) {
        /// The absolute root of `max_heap` is this index.
        const ROOT: usize = 0;

        if max_heap.is_empty() {
            return;
        }

        let mut current = ROOT;

        // Traverse down to leaf where the smallest possible value goes.
        while right_child(current) < max_heap.len() {
            if max_heap[right_child(current)] > max_heap[left_child(current)] {
                current = right_child(current);
            } else {
                current = left_child(current);
            }
        }

        if left_child(current) < max_heap.len() {
            current = left_child(current);
        }

        // Traverse upwards from that leaf to find where root should go.
        while max_heap[ROOT] > max_heap[current] {
            current = parent(current);
        }

        // Swap root into that position and propagate upwards.
        while current > ROOT {
            max_heap.swap(ROOT, current);
            current = parent(current);
        }
    }
}

/// Construct a binary max-heap.
mod max_heapify {
    use super::parent;
    use super::sift_down;
    use super::sift_up;

    /// Arrange `element` into max-heap (children less than parent) order.
    ///
    /// # Performance
    /// This method takes O(N) time and consumes O(1) memory.
    pub(super) fn bottom_up<T: Ord>(elements: &mut [T]) {
        if elements.len() <= 1 {
            return;
        }

        let last_parent = parent(elements.len() - 1) + 1;

        for parent in (0..=last_parent).rev() {
            // The children of `node` are already heap ordered, so sift down.
            sift_down::top_down(&mut elements[parent..]);
        }
    }

    /// Arrange `element` into max-heap (children less than parent) order.
    ///
    /// # Performance
    /// This method takes O(N * log N) time and consumes O(1) memory.
    pub(super) fn top_down<T: Ord>(elements: &mut [T]) {
        for leaf in 1..=elements.len() {
            // The ancestors of `leaf` are already heap ordered, so sift up.
            sift_up(&mut elements[..leaf]);
        }
    }
}

/// Sort `elements` via bottom-up heap sort.
///
/// Starting from lone elements which are themselves max-heap ordered,
/// iteratively join these subtrees by sifting down the element corresponding
/// to their parent until all elements are ordered. The max element (the root)
/// can then be swapped with the leaf with the highest index thereby placing it
/// in sorted order, sifting down the leaf to maintain ordering of the heap.
///
/// # Performance
/// This method takes O(N * log N) time and consumes O(1) memory.
///
/// # Examples
/// ```
/// use rust::algorithm::sort::comparison::heap::bottom_up;
///
/// let mut elements = [0, 5, 2, 3, 1, 4];
///
/// bottom_up(&mut elements);
///
/// assert_eq!(elements, [0, 1, 2, 3, 4, 5]);
/// ```
pub fn bottom_up<T: Ord>(elements: &mut [T]) {
    // Order `elements` in max-heap order, hence `elements[0]` is the greatest.
    max_heapify::bottom_up(elements);

    for end in (0..elements.len()).rev() {
        // Place the greatest element not yet sorted into sorted order.
        elements.swap(0, end);

        // Sift down the leaf into the max-heap (excluding sorted elements).
        sift_down::bottom_up(&mut elements[..end]);
    }
}

#[cfg(test)]
mod bottom_up {
    use super::bottom_up;

    #[test]
    fn empty() {
        let mut slice: [usize; 0] = [];
        bottom_up(&mut slice);
        assert_eq!(slice, []);
    }

    #[test]
    fn single() {
        let mut slice = [0];
        bottom_up(&mut slice);
        assert_eq!(slice, [0]);
    }

    #[test]
    fn sorted() {
        let mut slice = [0, 1];
        bottom_up(&mut slice);
        assert_eq!(slice, [0, 1]);
    }

    #[test]
    fn must_swap() {
        let mut slice = [1, 0];
        bottom_up(&mut slice);
        assert_eq!(slice, [0, 1]);
    }

    #[test]
    fn odd_length() {
        let mut slice = [3, 2, 1];
        bottom_up(&mut slice);
        assert_eq!(slice, [1, 2, 3]);
    }

    #[test]
    fn multiple_swap() {
        let mut slice = [2, 0, 3, 1];
        bottom_up(&mut slice);
        assert_eq!(slice, [0, 1, 2, 3]);
    }
}

/// Sort `elements` via bottom-up heap sort with inline sift-down optimization.
///
/// The implementation of [`bottom_up`] first creates a max-heap, and then
/// separately uses that structure to obtain elements in sorted order thereby
/// having two independent execution paths which ultimately invoke
/// [`sift_down`]. In contrast, this implementation combines both steps with
/// one shared execution path which would likely result in different runtime
/// characteristics given branch prediction and potential inline expansion.
///
/// /// # Performance
/// This method takes O(N * log N) time and consumes O(1) memory.
///
/// # Examples
/// ```
/// use rust::algorithm::sort::comparison::heap::bottom_up_inline;
///
/// let mut elements = [0, 5, 2, 3, 1, 4];
///
/// bottom_up_inline(&mut elements);
///
/// assert_eq!(elements, [0, 1, 2, 3, 4, 5]);
/// ```
pub fn bottom_up_inline<T: Ord>(elements: &mut [T]) {
    // This is the parent of the last element, hence it is the greatest index
    // of a node in the heap which has children. Since leaf elements within
    // `heap..` have a parent within `..=heap`, they will be heap-ordered by
    // the call to [`sift_down`] on their parent.
    let mut heap = elements.len() / 2;

    // Elements within `left_unsorted..` are sorted.
    let mut left_unsorted = elements.len();

    while left_unsorted > 1 {
        if heap > 0 {
            // The heap has yet to be constructed, and this call to
            // [`sift_down`] will add a subtree to the max-heap.
            heap -= 1;
        } else {
            // The heap has been constructed, hence `elements[0]` is the max
            // element which can therefore be swapped into sorted order, and
            // this call to [`sift_down`] will reorder the leaf into the heap.
            left_unsorted -= 1;
            elements.swap(left_unsorted, 0);
        }

        sift_down::top_down(&mut elements[heap..left_unsorted]);
    }
}

#[cfg(test)]
mod bottom_up_inline {
    use super::bottom_up_inline;

    #[test]
    fn empty() {
        let mut slice: [usize; 0] = [];
        bottom_up_inline(&mut slice);
        assert_eq!(slice, []);
    }

    #[test]
    fn single() {
        let mut slice = [0];
        bottom_up_inline(&mut slice);
        assert_eq!(slice, [0]);
    }

    #[test]
    fn sorted() {
        let mut slice = [0, 1];
        bottom_up_inline(&mut slice);
        assert_eq!(slice, [0, 1]);
    }

    #[test]
    fn must_swap() {
        let mut slice = [1, 0];
        bottom_up_inline(&mut slice);
        assert_eq!(slice, [0, 1]);
    }

    #[test]
    fn odd_length() {
        let mut slice = [3, 2, 1];
        bottom_up_inline(&mut slice);
        assert_eq!(slice, [1, 2, 3]);
    }

    #[test]
    fn multiple_swap() {
        let mut slice = [2, 0, 3, 1];
        bottom_up_inline(&mut slice);
        assert_eq!(slice, [0, 1, 2, 3]);
    }
}

/// Sift the last leaf of a `max_heap` up to the correct position.
///
/// Swap the leaf with its parent until the parent is greater.
///
/// # Performance
/// This method takes O(log N) time and consumes O(1) memory.
fn sift_up<T: Ord>(max_heap: &mut [T]) {
    if max_heap.len() <= 1 {
        return;
    }

    let mut current = max_heap.len() - 1;

    while current > 0 {
        if max_heap[parent(current)] < max_heap[current] {
            max_heap.swap(current, parent(current));
            current = parent(current);
        } else {
            return;
        }
    }
}

/// Sort a slice via top-down heap sort.
///
/// Create one max-heap at the start of the slice and then push each successive
/// element into it by 'sifting up'. Ordered elements are then popped from the
/// heap by swapping it with a leaf then 'sifting down' to preserve the heap.
///
/// # Examples
/// ```
/// use rust::algorithm::sort::comparison::heap::top_down;
/// let mut slice = [3, 2, 1];
/// top_down(&mut slice);
/// assert_eq!(slice, [1, 2, 3]);
/// ```
pub fn top_down<T>(slice: &mut [T])
where
    T: Ord,
{
    max_heapify::top_down(slice);

    for end in (0..slice.len()).rev() {
        // max-heap implies the root node is the greatest in the collection,
        // pop it from the max-heap by swapping it with the last element.
        slice.swap(0, end);

        // push the new root into the shrunk max-heap excluding sorted element.
        sift_down::top_down(&mut slice[..end]);
    }
}

#[cfg(test)]
mod top_down {
    use super::top_down;

    #[test]
    fn empty() {
        let mut slice: [usize; 0] = [];
        top_down(&mut slice);
        assert_eq!(slice, []);
    }

    #[test]
    fn single() {
        let mut slice = [0];
        top_down(&mut slice);
        assert_eq!(slice, [0]);
    }

    #[test]
    fn sorted() {
        let mut slice = [0, 1];
        top_down(&mut slice);
        assert_eq!(slice, [0, 1]);
    }

    #[test]
    fn must_swap() {
        let mut slice = [1, 0];
        top_down(&mut slice);
        assert_eq!(slice, [0, 1]);
    }

    #[test]
    fn odd_length() {
        let mut slice = [3, 2, 1];
        top_down(&mut slice);
        assert_eq!(slice, [1, 2, 3]);
    }

    #[test]
    fn multiple_swap() {
        let mut slice = [2, 0, 3, 1];
        top_down(&mut slice);
        assert_eq!(slice, [0, 1, 2, 3]);
    }
}
