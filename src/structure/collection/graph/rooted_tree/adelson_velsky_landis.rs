//! Implementation of [`AdelsonVelskyLandis`].

use super::Collection;
use super::Graph;
use super::Directed;
use super::RootedTree;

/// A self-balancing binary search tree.
///
/// Unlike an unbalanced binary search tree, the heights of the two child
/// subtrees of any [`Node`] differ by at most one thereby minimizing the
/// height of the overall tree and providing optimal lookup/search performance.
///
/// See Also: [Wikipedia](https://en.wikipedia.org/wiki/AVL_tree).
pub struct AdelsonVelsoLandis<T> {
    /// The [`Node`] that is defined as the root.
    root: Option<Box<Node<T>>>,
}

impl<T: Ord> AdelsonVelsoLandis<T> {
    /// Add a new [`Node`] with value `element`.
    ///
    /// # Errors
    /// Yields the `element` if an equivalent one is already contained,
    /// alongside a mutable reference to the contained equivalent element.
    ///
    /// # Panics
    /// The Rust runtime might panic or otherwise abort if allocation fails.
    ///
    /// # Performance
    /// This method takes O(log N) time and consumes O(1) memory.
    ///
    /// # Examples
    /// ```
    /// todo!()
    /// ```
    pub fn insert(&mut self, element: T) -> Result<&mut T, (T, &mut T)> {
        let mut parent_ptr = None;

        let mut current = &mut self.root;

        while let &mut Some(ref mut parent) = current {
            parent_ptr = Some(core::ptr::from_mut(parent.as_mut()));

            current = match element.cmp(&parent.element) {
                core::cmp::Ordering::Less => &mut parent.left,
                core::cmp::Ordering::Greater => &mut parent.right,
                core::cmp::Ordering::Equal => return Err((element, &mut parent.element)),
            };
        }

        let node = Box::new(Node {
            element,
            parent: parent_ptr,
            left: None,
            right: None,
        });

        let node = current.insert(node);

        Ok(&mut node.element)
    }
}

/// An instantiated element within a [`AdelsonVelskyLandis`].
struct Node<T> {
    /// The underlying element.
    element: T,

    /// The parent of `self`, if any.
    parent: Option<*mut Node<T>>,

    /// The left child, if any.
    left: Option<Box<Node<T>>>,

    /// The right child, if any.
    right: Option<Box<Node<T>>>,
}

#[cfg(test)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::assertions_on_result_states
)]
mod test {
    use super::*;

    mod method {
        use super::*;

        mod insert {
            use super::*;

            #[test]
            fn adds_element() {
                let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                assert!(instance.insert(12345).is_ok());

                assert!(instance.root.is_some());
            }

            #[test]
            fn initializes_element() {
                let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                let expected = 12345;

                assert!(instance.insert(expected).is_ok());

                assert!(instance.root.is_some_and(|node| node.element == expected));
            }

            #[test]
            fn yields_element() {
                let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                let mut expected = 12345;

                assert!(instance.insert(expected).is_ok_and(|actual| actual == &mut expected));
            }

            #[test]
            fn into_left_branch_when_less() {
                let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                // Insert the root node.
                assert!(instance.insert(0).is_ok());

                // Insert the child node that is less than root.
                let expected = -1;
                assert!(instance.insert(expected).is_ok());

                assert!(instance.root.is_some_and(|root| root.left.is_some_and(|left| left.element == expected)));
            }

            #[test]
            fn into_right_branch_when_greater() {
                let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                // Insert the root node.
                assert!(instance.insert(0).is_ok());

                // Insert the child node that is greater than root.
                let expected = 1;
                assert!(instance.insert(expected).is_ok());

                assert!(instance.root.is_some_and(|root| root.right.is_some_and(|right| right.element == expected)));
            }

            #[test]
            fn parent_is_none_when_root() {
                let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                assert!(instance.insert(12345).is_ok());

                assert!(instance.root.is_some_and(|root| root.parent.is_none()));
            }

            #[test]
            fn parent_is_some_when_child() {
                let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                // Insert root.
                assert!(instance.insert(0).is_ok());

                // Insert left child.
                assert!(instance.insert(-1).is_ok());

                // Insert right child.
                assert!(instance.insert(1).is_ok());

                let ptr = core::ptr::NonNull::from(instance.root.as_deref().unwrap());

                assert!(instance.root.as_ref().is_some_and(|root| root.left.as_ref().is_some_and(|left| left.parent.is_some_and(|parent| parent == ptr))));
                assert!(instance.root.as_ref().is_some_and(|root| root.right.as_ref().is_some_and(|right| right.parent.is_some_and(|parent| parent == ptr))));
            }

            mod errors {
                use super::*;

                #[test]
                fn when_equivalent_element_already_contained() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    assert!(instance.insert(12345).is_ok());

                    assert!(instance.insert(12345).is_err());
                }

                #[test]
                fn yields_new_element() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    assert!(instance.insert(12345).is_ok());

                    assert!(instance.insert(12345).is_err_and(|error| error.0 == 12345));
                }

                #[test]
                fn yields_existing_equivalent_element() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    assert!(instance.insert(12345).is_ok());

                    assert!(instance.insert(12345).is_err_and(|error| error.1 == &mut 12345));
                }
            }

            mod balance_factor {
                use super::*;

                #[test]
                fn inserted_node_is_balanced() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    assert!(instance.insert(0).is_ok());

                    assert!(instance.root.is_some_and(|node| node.balance_factor == BalanceFactor::Balanced));
                }

                #[test]
                fn left_balanced_when_left_branch() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    // Insert the root.
                    assert!(instance.insert(0).is_ok());

                    // Insert a left child.
                    assert!(instance.insert(-1).is_ok());

                    assert!(instance.root.is_some_and(|node| node.balance_factor == BalanceFactor::Left));
                }

                #[test]
                fn right_balanced_when_right_branch() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    // Insert the root.
                    assert!(instance.insert(0).is_ok());

                    // Insert a right child.
                    assert!(instance.insert(1).is_ok());

                    assert!(instance.root.is_some_and(|node| node.balance_factor == BalanceFactor::Right));
                }

                #[test]
                fn balanced_when_left_branch_and_then_right_branch() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    // Insert the root.
                    assert!(instance.insert(0).is_ok());

                    // Insert a right child.
                    assert!(instance.insert(1).is_ok());

                    // Insert a left child.
                    assert!(instance.insert(-1).is_ok());

                    assert!(instance.root.is_some_and(|node| node.balance_factor == BalanceFactor::Balanced));
                }

                #[test]
                fn balanced_when_right_branch_and_then_left_branch() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    // Insert the root.
                    assert!(instance.insert(0).is_ok());

                    // Insert a left child.
                    assert!(instance.insert(-1).is_ok());

                    // Insert a right child.
                    assert!(instance.insert(1).is_ok());

                    assert!(instance.root.is_some_and(|node| node.balance_factor == BalanceFactor::Balanced));
                }
            }

            mod left_rotate {
                use super::*;

                /// Insert elements to induce a left rotation.
                ///
                /// This is a helper function which normalizes the [`Node`]
                /// tested within this module.
                ///
                /// The insertions create the following structure
                ///
                ///  0
                /// / \
                ///   1
                ///  / \
                ///    2
                ///
                /// which should be rebalanced via a left-rotation into below
                ///
                ///   1
                ///  / \
                ///  0 2
                fn insert_elements(instance: &mut AdelsonVelsoLandis<i32>) {
                    // Insert the root.
                    assert!(instance.insert(0).is_ok());

                    // Insert a right child.
                    assert!(instance.insert(1).is_ok());

                    // Insert a right grandchild.
                    assert!(instance.insert(2).is_ok());
                }

                #[test]
                fn rotates_elements() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.element == 1));
                    assert!(instance.root.as_ref().is_some_and(|root| root.left.as_ref().is_some_and(|left| left.element == 0)));
                    assert!(instance.root.as_ref().is_some_and(|root| root.right.as_ref().is_some_and(|right| right.element == 2)));
                }

                #[test]
                fn updates_parents() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.parent.is_none()));

                    let ptr = core::ptr::NonNull::from(instance.root.as_deref().unwrap());

                    assert!(instance.root.as_deref().is_some_and(|root| root.left.as_deref().is_some_and(|left| left.parent.is_some_and(|parent| parent == ptr))));
                    assert!(instance.root.as_deref().is_some_and(|root| root.right.as_deref().is_some_and(|right| right.parent.is_some_and(|parent| parent == ptr))));
                }
            }

            mod right_rotate {
                use super::*;

                /// Insert elements to induce a right rotation.
                ///
                /// This is a helper function which normalizes the [`Node`]
                /// tested within this module.
                ///
                /// The insertions create the following structure
                ///
                ///     0
                ///    / \
                ///   -1
                ///   / \
                ///  -2
                ///
                /// which should be rebalanced via a left-rotation into below
                ///
                ///   -1
                ///  / \
                /// -2 0
                fn insert_elements(instance: &mut AdelsonVelsoLandis<i32>) {
                    // Insert the root.
                    assert!(instance.insert(0).is_ok());

                    // Insert a left child.
                    assert!(instance.insert(-1).is_ok());

                    // Insert a left grandchild.
                    assert!(instance.insert(-2).is_ok());
                }

                #[test]
                fn rotates_elements() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.element == -1));
                    assert!(instance.root.as_ref().is_some_and(|root| root.left.as_ref().is_some_and(|left| left.element == -1)));
                    assert!(instance.root.as_ref().is_some_and(|root| root.right.as_ref().is_some_and(|right| right.element == 0)));
                }

                #[test]
                fn updates_parents() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.parent.is_none()));

                    let ptr = core::ptr::NonNull::from(instance.root.as_deref().unwrap());

                    assert!(instance.root.as_deref().is_some_and(|root| root.left.as_deref().is_some_and(|left| left.parent.is_some_and(|parent| parent == ptr))));
                    assert!(instance.root.as_deref().is_some_and(|root| root.right.as_deref().is_some_and(|right| right.parent.is_some_and(|parent| parent == ptr))));
                }
            }

            mod left_right_rotate {
                use super::*;

                /// Insert elements to induce a left-right rotation.
                ///
                /// This is a helper function which normalizes the [`Node`]
                /// tested within this module.
                ///
                /// The insertions create the following structure
                ///
                ///     3
                ///    / \
                ///    1
                ///   / \
                ///     2
                ///
                /// which should be rebalanced via a left-right-rotation into below
                ///
                ///   2
                ///  / \
                ///  1 3
                fn insert_elements(instance: &mut AdelsonVelsoLandis<i32>) {
                    // Insert the root.
                    assert!(instance.insert(3).is_ok());

                    // Insert a left child.
                    assert!(instance.insert(1).is_ok());

                    // Insert a right grandchild.
                    assert!(instance.insert(2).is_ok());
                }

                #[test]
                fn rotates_elements() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.element == 2));
                    assert!(instance.root.as_ref().is_some_and(|root| root.left.as_ref().is_some_and(|left| left.element == 1)));
                    assert!(instance.root.as_ref().is_some_and(|root| root.right.as_ref().is_some_and(|right| right.element == 3)));
                }

                #[test]
                fn updates_parents() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.parent.is_none()));

                    let ptr = core::ptr::NonNull::from(instance.root.as_deref().unwrap());

                    assert!(instance.root.as_deref().is_some_and(|root| root.left.as_deref().is_some_and(|left| left.parent.is_some_and(|parent| parent == ptr))));
                    assert!(instance.root.as_deref().is_some_and(|root| root.right.as_deref().is_some_and(|right| right.parent.is_some_and(|parent| parent == ptr))));
                }
            }

            mod right_left_rotate {
                use super::*;

                /// Insert elements to induce a right-left rotation.
                ///
                /// This is a helper function which normalizes the [`Node`]
                /// tested within this module.
                ///
                /// The insertions create the following structure
                ///
                ///     2
                ///    / \
                ///      1
                ///     / \
                ///     0
                ///
                /// which should be rebalanced via a right-left-rotation into below
                ///
                ///   0
                ///  / \
                ///  1 2
                fn insert_elements(instance: &mut AdelsonVelsoLandis<i32>) {
                    // Insert the root.
                    assert!(instance.insert(2).is_ok());

                    // Insert a right child.
                    assert!(instance.insert(1).is_ok());

                    // Insert a left grandchild.
                    assert!(instance.insert(0).is_ok());
                }

                #[test]
                fn rotates_elements() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.element == 0));
                    assert!(instance.root.as_ref().is_some_and(|root| root.left.as_ref().is_some_and(|left| left.element == 1)));
                    assert!(instance.root.as_ref().is_some_and(|root| root.right.as_ref().is_some_and(|right| right.element == 2)));
                }

                #[test]
                fn updates_parents() {
                    let mut instance = AdelsonVelsoLandis::<i32> { root: None };

                    insert_elements(&mut instance);

                    assert!(instance.root.as_ref().is_some_and(|root| root.parent.is_none()));

                    let ptr = core::ptr::NonNull::from(instance.root.as_deref().unwrap());

                    assert!(instance.root.as_deref().is_some_and(|root| root.left.as_deref().is_some_and(|left| left.parent.is_some_and(|parent| parent == ptr))));
                    assert!(instance.root.as_deref().is_some_and(|root| root.right.as_deref().is_some_and(|right| right.parent.is_some_and(|parent| parent == ptr))));
                }
            }
        }
    }
}
