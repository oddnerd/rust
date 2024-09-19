//! Implementation of [`AdelsonVelskyLandis`].

use super::Binary;
use super::Tree;
use super::Graph;
use super::Collection;

/// TODO: [Wikipedia](https://en.wikipedia.org/wiki/AVL_tree)
pub struct AdelsonVelskyLandis<T> {
    /// The root node of this tree.
    _root: Option<Node<T>>
}

impl<T> core::fmt::Debug for AdelsonVelskyLandis<T> {
    /// List the elements contained and their hierarchical order.
    ///
    /// TODO: performance; time and memory complexity.
    /// TODO: example use case.
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!("how to format the graph into a string?")
    }
}

impl<T> Binary for AdelsonVelskyLandis<T> {}

impl<T> Tree for AdelsonVelskyLandis<T> {}

impl<T> Graph for AdelsonVelskyLandis<T> {
    type Node = Node<Self::Element>;

    type Edge<'a> = Edge<'a, Self::Element> where Self::Element: 'a;
}

impl<T> Collection for AdelsonVelskyLandis<T> {
    type Element = T;

    fn count(&self) -> usize {
        todo!()
    }
}

/// An instantiated element with an [`AdelsonVelskyLandis`].
pub struct Node<T> {
    /// The underlying element.
    _element: T,

    /// The left child branch, if there is one.
    _left: Option<Box<Node<T>>>,

    /// The right child branch, if there is one.
    _right: Option<Box<Node<T>>>,
}

/// A link between two [`Node`] in a [`AdelsonVelskyLandis`].
pub struct Edge<'a, T: 'a> (&'a Node<T>, &'a Node<T>);
