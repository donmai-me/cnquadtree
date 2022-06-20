use crate::location::{Cardinality, Location};
use num_traits::{FromPrimitive, NumAssign, NumOps, ToPrimitive};

pub trait RegionQuadtreeNode<T>: PartialEq {
    type Index: Clone;
    type Unit: Copy
        + Clone
        + PartialOrd
        + PartialEq
        + NumAssign
        + ToPrimitive
        + NumOps
        + FromPrimitive;

    /// Returns the parent node index of a node if it exists.
    fn get_parent_index(&self) -> Option<Self::Index>;
    /// Returns true if the node has a parent.
    fn has_parent(&self) -> bool {
        self.get_parent_index().is_some()
    }
    /// Return an array of child indices if it exists.
    fn get_children_index(&self) -> Option<[Self::Index; 4]>;
    /// Return a child index at the specified location if it exists.
    fn get_child_index(&self, location: Location) -> Option<Self::Index> {
        self.get_children_index()
            .and_then(|children| Some(children[location as usize].clone()))
    }
    /// Returns true if the node has children.
    fn has_children(&self) -> bool {
        self.get_children_index().is_some()
    }
    /// Returns true if the node is a leaf.
    fn is_leaf(&self) -> bool {
        self.get_children_index().is_none()
    }
    /// Returns a shared reference to the node's item.
    fn get_item(&self) -> &T;
    /// Returns a unique reference to the node's item.
    fn get_item_mut(&mut self) -> &mut T;
    /// Consumes the node and returns its item. Make sure that the node no longer has any
    /// children and no neighbors pointing to it.
    fn pop(self) -> T;
    /// Returns the node's level with respect to the tree's root node (level 0)
    fn level(&self) -> usize;
    /// Returns the four cardinal neighbor indices of the node. In the following order:
    /// West, North, East, South. An index may be none if the node is a border node.
    fn get_cardinal_neighbors_index(&self) -> [Option<Self::Index>; 4];
    /// Returns the cardinal neighbor index at the specified direction of a node.
    /// An index may be none if the node is a border node at the specified direction.
    fn get_cardinal_neighbor_index(&self, direction: Cardinality) -> Option<Self::Index> {
        self.get_cardinal_neighbors_index()[direction as usize].clone()
    }

    fn has_neighbor(&self, direction: Cardinality) -> bool {
        self.get_cardinal_neighbor_index(direction).is_some()
    }
    fn update_neighbor(&mut self, new_neighbor: Option<Self::Index>, direction: Cardinality);
    fn update_neighbors(&mut self, new_neighbors: [Option<Self::Index>; 4]) {
        for (direction, new_neighbor) in new_neighbors.into_iter().enumerate() {
            self.update_neighbor(new_neighbor, Cardinality::try_from(direction).unwrap());
        }
    }
    fn update_children(&mut self, new_children: Option<[Self::Index; 4]>);
    fn get_bounds(&self) -> (Self::Unit, Self::Unit, Self::Unit, Self::Unit);
    fn point_in(&self, point: (Self::Unit, Self::Unit)) -> bool {
        let bounds = self.get_bounds();
        (bounds.0 <= point.0 && point.0 < bounds.2) && (bounds.1 <= point.1 && point.1 < bounds.3)
    }
}

pub struct CNNode<T, I, S = u32>
where
    S: Copy + Clone + PartialOrd + PartialEq + NumAssign + ToPrimitive + NumOps + FromPrimitive,
    I: Copy + Clone,
{
    item: T,
    layer: usize,
    // min x, min y, max x, max y
    bounds: (S, S, S, S),
    parent: Option<I>,
    /// Cardinal neighbors in the following order: West, North, East, South.
    /// A neighbor is None if it's a border.
    neighbors: [Option<I>; 4],
    /// Children in the following order: NorthWest, NorthEast, SouthWest, SouthEast.
    children: Option<[I; 4]>,
}

impl<T, I, S> PartialEq for CNNode<T, I, S>
where
    S: Copy + Clone + PartialOrd + PartialEq + NumAssign + ToPrimitive + NumOps + FromPrimitive,
    I: Copy + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.bounds == other.bounds
    }
}

impl<T, S, I> RegionQuadtreeNode<T> for CNNode<T, I, S>
where
    S: Copy + Clone + PartialOrd + PartialEq + NumAssign + ToPrimitive + NumOps + FromPrimitive,
    I: Copy + Clone,
{
    type Index = I;
    type Unit = S;

    #[inline]
    fn get_parent_index(&self) -> Option<Self::Index> {
        self.parent
    }

    #[inline]
    fn get_children_index(&self) -> Option<[Self::Index; 4]> {
        self.children
    }

    #[inline]
    fn get_item(&self) -> &T {
        &self.item
    }

    #[inline]
    fn get_item_mut(&mut self) -> &mut T {
        &mut self.item
    }

    fn pop(self) -> T {
        self.item
    }

    #[inline]
    fn level(&self) -> usize {
        self.layer
    }

    #[inline]
    fn get_cardinal_neighbors_index(&self) -> [Option<Self::Index>; 4] {
        self.neighbors
    }

    #[inline]
    fn update_neighbor(&mut self, new_neighbor: Option<Self::Index>, direction: Cardinality) {
        self.neighbors[direction as usize] = new_neighbor;
    }

    #[inline]
    fn update_children(&mut self, new_children: Option<[Self::Index; 4]>) {
        self.children = new_children;
    }

    #[inline]
    fn get_bounds(&self) -> (Self::Unit, Self::Unit, Self::Unit, Self::Unit) {
        self.bounds
    }
}

impl<T, S, I> CNNode<T, I, S>
where
    S: Copy + Clone + PartialOrd + NumAssign + ToPrimitive + NumOps + FromPrimitive,
    I: Copy + Clone,
{
    pub(crate) fn new(item: T, layer: usize, bounds: (S, S, S, S), parent: Option<I>) -> Self {
        Self {
            item,
            layer,
            bounds,
            parent,
            neighbors: [None; 4],
            children: None,
        }
    }

    pub(crate) fn pop(self) -> T {
        self.item
    }
}
