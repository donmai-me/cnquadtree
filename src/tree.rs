use crate::location::{Cardinality, Location};
use crate::node::RegionQuadtreeNode;
use std::fmt::Debug;

use thiserror::Error;

pub trait RegionQuadtree<T> {
    type Index: Clone;
    type Node: RegionQuadtreeNode<T, Index = Self::Index>;

    /// Returns a shared ref to the node if index is valid. Otherwise, returns None.
    fn get_node(&self, index: Self::Index) -> Option<&Self::Node>;
    /// Returns a unique ref to the node if index is valid. Otherwise returns None.
    fn get_node_mut(&mut self, index: Self::Index) -> Option<&mut Self::Node>;
    /// Returns the root node's index.
    fn get_root(&self) -> Self::Index;
    fn map_children<O>(
        &mut self,
        index: Self::Index,
        f: fn(&mut Self::Node) -> O,
    ) -> Option<[O; 4]> {
        let children_index = self.get_node(index)?.get_children_index()?;
        Some(children_index.map(|c| f(self.get_node_mut(c).expect("failed to get child node"))))
    }
    fn get_neighbors(
        &self,
        index: Self::Index,
        direction: Cardinality,
    ) -> Option<Vec<Self::Index>> {
        let node = self.get_node(index)?;
        let first_neighbor_index = node.get_cardinal_neighbor_index(direction)?;
        let first_neighbor = self.get_node(first_neighbor_index.clone())?;

        // Allocate 2^(first_neighbor.layer - node.layer) if first_neighbor is further down the tree
        // Otherwise only allocate one slot.
        let capacity = {
            if node.level() < first_neighbor.level() {
                2_usize.pow((first_neighbor.level() - node.level()) as u32)
            } else {
                1
            }
        };

        let mut result = Vec::with_capacity(capacity);

        let mut neighbor = first_neighbor;
        let mut neighbor_index = first_neighbor_index;
        // While succeeding neighbor is still the cardinal neighbor of node in the opposite direction
        loop {
            result.push(neighbor_index.clone());
            neighbor_index = match neighbor.get_cardinal_neighbor_index(direction.next_neighbor()) {
                None => break,
                Some(n) => n,
            };
            neighbor = match self.get_node(neighbor_index.clone()) {
                None => break,
                Some(n) => n,
            };

            // neighbor's cardinal neighbor in the opposite direction of the given direction
            let opposite_side_index =
                match neighbor.get_cardinal_neighbor_index(direction.opposite()) {
                    None => break,
                    Some(i) => i,
                };
            let opposite_side = match self.get_node(opposite_side_index) {
                None => break,
                Some(x) => x,
            };

            if neighbor.level() >= node.level() || node != opposite_side {
                break;
            }
        }

        Some(result)
    }
    fn map_neighbors<O>(
        &mut self,
        index: Self::Index,
        f: fn(&mut Self::Node) -> O,
        direction: Cardinality,
    ) -> Option<Vec<O>> {
        let neighbors = self.get_neighbors(index, direction)?;
        Some(
            neighbors
                .into_iter()
                .map(|n| f(self.get_node_mut(n).expect("failed to get neighbor node")))
                .collect(),
        )
    }
    fn subdivide(
        &mut self,
        index: Self::Index,
        items: [T; 4],
    ) -> Result<[Self::Index; 4], SubdivideError<T>>;
    fn pop_children(&mut self, index: Self::Index) -> Option<[T; 4]>;
    fn location_among_siblings(&self, index: Self::Index) -> Option<Location> {
        let node = self.get_node(index)?;
        let parent = self.get_node(node.get_parent_index()?)?;
        let children = parent.get_children_index()?;
        Some(
            children
                .into_iter()
                .position(|x| self.get_node(x.clone()) == Some(node))?
                .try_into()
                .unwrap(),
        )
    }
    fn point_locate(
        &self,
        point: (
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
        ),
    ) -> Option<Self::Index>;
    fn region_locate(
        &self,
        region: (
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
        ),
    ) -> Option<Vec<Self::Index>>;
}

/// Error type for quadtree subdivision.
/// `items` are the passed items to the subdivision function.
#[derive(Debug, Error)]
pub struct SubdivideError<T> {
    pub items: [T; 4],
    pub source: SubdivideErrorEnum,
}

/// Enum error type for quadtree subdivision.
#[derive(Debug, Error, Copy, Clone)]
pub enum SubdivideErrorEnum {
    #[error("node index is invalid")]
    InvalidIndex,
    #[error("node is already subdivided")]
    AlreadySubdivided,
}

pub fn find_cardinal_neighbor<T, U>(
    tree: &T,
    child_layer: usize,
    direction: Cardinality,
    inherited_neighbor: T::Index,
) -> Option<T::Index>
where
    T: RegionQuadtree<U>,
{
    // TODO: Rewrite using bitwise operations
    let mut layers = vec![0_usize];
    let mut current_neighbor = tree.get_node(inherited_neighbor.clone())?;
    let mut current_neighbor_index = inherited_neighbor.clone();

    while layers[0] != 0 {
        let index = current_neighbor.level().saturating_sub(child_layer);
        if index >= layers.len() {
            layers.resize(index + 1, 0);
        }

        layers[index] += 1;

        for index in (0..layers.len()).rev() {
            if layers[index] >= 2 && index != 0 {
                layers[index] = 0;
                layers[index - 1] += 1;
            }
        }

        current_neighbor_index =
            current_neighbor.get_cardinal_neighbor_index(direction.next_neighbor())?;
        current_neighbor = tree.get_node(current_neighbor_index.clone())?;
    }

    // Return the next neighbor
    current_neighbor_index =
        current_neighbor.get_cardinal_neighbor_index(direction.next_neighbor())?;

    Some(current_neighbor_index)
}
