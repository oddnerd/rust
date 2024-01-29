pub fn top_down<T>(slice: &mut [T], auxiliary: &mut [T]) -> ()
where
    T: Ord + Clone,
{
    if slice.len() > 1 {
        assert!(slice.len() == auxiliary.len());
        let (left_auxiliary, right_auxiliary) = auxiliary.split_at_mut(auxiliary.len() / 2);

        let (left_slice, right_slice) = slice.split_at_mut(slice.len() / 2);

        // auxiliary becomes the sorted left/right slices to be merged,
        // alternate input/auxiliary to avoid additional clone
        top_down(left_auxiliary, left_slice);
        top_down(right_auxiliary, right_slice);

        let merger =
            crate::algorithm::merge::MergeIter::new(left_auxiliary.iter(), right_auxiliary.iter());

        std::iter::zip(slice, merger).for_each(|(old, new)| {
            *old = new.clone();
        });
    }
}

pub fn bottom_up<T>(slice: &mut [T], auxiliary: &mut [T]) -> ()
where
    T: Ord + Clone + std::fmt::Debug,
{
    let mut width: usize = 1;
    loop {
        if width < slice.len() {
            fn min<T: Ord>(first: T, second: T) -> T {
                if first < second {
                    first
                } else {
                    second
                }
            }

            let mut i: usize = 0;
            loop {
                if i < slice.len() {
                    let right = min(i + width, slice.len());
                    let end = min(i + 2 * width, slice.len());

                    let subslice = &mut auxiliary[i..end];
                    let (first, second) = subslice.split_at_mut(right - i);

                    let merger = crate::algorithm::merge::MergeIter::new(
                        first.iter(),
                        second.iter(),
                    );

                    std::iter::zip(slice[i..end].iter_mut(), merger).for_each(|(old, new)| {
                        *old = new.clone();
                    });
                } else {
                    break;
                }
                i += width * 2
            }
        } else {
            break;
        }
        width *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_down_empty() {
        let mut slice: [usize; 0] = [];
        let mut auxiliary = slice.to_vec();
        top_down(&mut slice, &mut auxiliary);
        assert_eq!(slice, []);
    }

    #[test]
    fn top_down_one() {
        let mut slice = [0];
        let mut auxiliary = slice.to_vec();
        top_down(&mut slice, &mut auxiliary);
        assert_eq!(slice, [0]);
    }

    #[test]
    fn top_down_two() {
        let mut slice = [2, 1];
        let mut auxiliary = slice.to_vec();
        top_down(&mut slice, &mut auxiliary);
        assert_eq!(slice, [1, 2]);
    }

    #[test]
    fn top_down_multiple() {
        let mut slice = [3, 2, 1];
        let mut auxiliary = slice.to_vec();
        top_down(&mut slice, &mut auxiliary);
        assert_eq!(slice, [1, 2, 3]);
    }

    #[test]
    fn bottom_up_empty() {
        let mut slice: [usize; 0] = [];
        let mut auxiliary = slice.to_vec();
        bottom_up(&mut slice, &mut auxiliary);
        assert_eq!(slice, []);
    }

    #[test]
    fn bottom_up_one() {
        let mut slice = [0];
        let mut auxiliary = slice.to_vec();
        bottom_up(&mut slice, &mut auxiliary);
        assert_eq!(slice, [0]);
    }

    #[test]
    fn bottom_up_two() {
        let mut slice = [2, 1];
        let mut auxiliary = slice.to_vec();
        bottom_up(&mut slice, &mut auxiliary);
        assert_eq!(slice, [1, 2]);
    }

    #[test]
    fn bottom_up_multiple() {
        let mut slice: Vec<i32> = (0..16).collect();
        let copy = slice.clone();
        let mut auxiliary = slice.to_vec();
        bottom_up(&mut slice, &mut auxiliary);
        assert_eq!(slice, copy);
    }
}
