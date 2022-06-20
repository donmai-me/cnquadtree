use crate::location::Cardinality;
use crate::node::{CNNode, RegionQuadtreeNode};
use crate::tree::{find_cardinal_neighbor, RegionQuadtree, SubdivideError, SubdivideErrorEnum};
use num_traits::{FromPrimitive, NumAssign, NumOps, ToPrimitive};
use slotmap::{DefaultKey, SlotMap};

pub struct CNQuadtree<T, S = u32>
where
    S: Copy + Clone + PartialOrd + PartialEq + NumAssign + ToPrimitive + NumOps + FromPrimitive,
{
    store: SlotMap<DefaultKey, CNNode<T, DefaultKey, S>>,
    root_key: DefaultKey,
    layers: Vec<usize>,
}

impl<T, S> CNQuadtree<T, S>
where
    S: Copy + Clone + PartialOrd + PartialEq + NumAssign + ToPrimitive + NumOps + FromPrimitive,
{
    pub fn new(item: T, bounds: (S, S, S, S)) -> Self {
        let root_node = CNNode::<T, DefaultKey, S>::new(item, 0, bounds, None);

        let mut store = SlotMap::new();
        let root_key = store.insert(root_node);

        Self {
            store,
            root_key,
            layers: vec![1],
        }
    }

    fn get_children_cardinal_neighbors(
        &self,
        cardinal_neighbor: Option<DefaultKey>,
        parent_layer: usize,
        cardinality: Cardinality,
    ) -> (Option<DefaultKey>, Option<DefaultKey>) {
        match cardinal_neighbor {
            None => (None, None),
            Some(inherited_neighbor) => (
                Some(inherited_neighbor),
                find_cardinal_neighbor::<CNQuadtree<T, S>, T>(
                    &self,
                    parent_layer + 1,
                    cardinality,
                    inherited_neighbor,
                ),
            ),
        }
    }

    fn get_and_update_children_neighbors(
        &mut self,
        first_child: DefaultKey,
        second_child: DefaultKey,
        parent: DefaultKey,
        cardinality: Cardinality,
    ) -> Option<DefaultKey> {
        let mut neighbors = self.get_neighbors(first_child, cardinality)?;
        let mut other_neighbors = self.get_neighbors(second_child, cardinality)?;
        neighbors.append(&mut other_neighbors);

        for neighbor in neighbors.iter() {
            self.get_node_mut(*neighbor)
                .unwrap()
                .update_neighbor(Some(parent), cardinality.opposite());
        }

        Some(neighbors[0])
    }

    fn update_neighbors_to_children(
        &mut self,
        neighbors: Option<Vec<DefaultKey>>,
        first_child: DefaultKey,
        second_child: DefaultKey,
        second_child_cardinal_neighbor: Option<DefaultKey>,
        cardinality: Cardinality,
    ) {
        match neighbors {
            None => {}
            Some(neighbors) => {
                let mut new_neighbor = Some(first_child);
                for neighbor in neighbors {
                    if neighbor == second_child_cardinal_neighbor.unwrap() {
                        new_neighbor = Some(second_child);
                    }
                    self.get_node_mut(neighbor)
                        .unwrap()
                        .update_neighbor(new_neighbor.clone(), cardinality);
                }
            }
        }
    }

    #[inline]
    fn get_max_level(&self) -> usize {
        self.layers
            .iter()
            .enumerate()
            .filter_map(|(layer, &num)| if num > 0 { Some(layer) } else { None })
            .max()
            .unwrap()
    }
}

impl<T, S> RegionQuadtree<T> for CNQuadtree<T, S>
where
    S: Copy + Clone + PartialOrd + PartialEq + NumAssign + ToPrimitive + NumOps + FromPrimitive,
{
    type Index = DefaultKey;
    type Node = CNNode<T, DefaultKey, S>;

    fn get_node(&self, index: Self::Index) -> Option<&Self::Node> {
        self.store.get(index)
    }

    fn get_node_mut(&mut self, index: Self::Index) -> Option<&mut Self::Node> {
        self.store.get_mut(index)
    }

    fn get_root(&self) -> Self::Index {
        self.root_key
    }

    fn subdivide(
        &mut self,
        index: Self::Index,
        items: [T; 4],
    ) -> Result<[Self::Index; 4], SubdivideError<T>> {
        let (parent_layer, bounds) = match self.get_node(index) {
            Some(x) if !x.has_children() => (x.level(), x.get_bounds()),
            Some(x) if x.has_children() => {
                return Err(SubdivideError {
                    items,
                    source: SubdivideErrorEnum::AlreadySubdivided,
                })
            }
            _ => {
                return Err(SubdivideError {
                    items,
                    source: SubdivideErrorEnum::InvalidIndex,
                })
            }
        };

        let [nw_item, ne_item, sw_item, se_item] = items;
        let (left, top, right, bottom) = bounds;

        let x_middle = (left + right) / S::from_i64(2).unwrap();
        let y_middle = (top + bottom) / S::from_i64(2).unwrap();

        // Get neighbors.
        let w_neighbors = self.get_neighbors(index, Cardinality::West);
        let n_neighbors = self.get_neighbors(index, Cardinality::North);
        let e_neighbors = self.get_neighbors(index, Cardinality::East);
        let s_neighbors = self.get_neighbors(index, Cardinality::South);

        // Get inherited and calculated non-sibling cardinal neighbors.
        let (ne_n_neighbor, nw_n_neighbor) = self.get_children_cardinal_neighbors(
            n_neighbors.as_ref().and_then(|n| n.first().cloned()),
            parent_layer,
            Cardinality::North,
        );
        let (sw_w_neighbor, nw_w_neighbor) = self.get_children_cardinal_neighbors(
            w_neighbors.as_ref().and_then(|n| n.first().cloned()),
            parent_layer,
            Cardinality::West,
        );
        let (sw_s_neighbor, se_s_neighbor) = self.get_children_cardinal_neighbors(
            s_neighbors.as_ref().and_then(|n| n.first().cloned()),
            parent_layer,
            Cardinality::South,
        );
        let (ne_e_neighbor, se_e_neighbor) = self.get_children_cardinal_neighbors(
            e_neighbors.as_ref().and_then(|n| n.first().cloned()),
            parent_layer,
            Cardinality::East,
        );

        // Create child nodes.
        let nw_node = CNNode::<T, DefaultKey, S>::new(
            nw_item,
            parent_layer + 1,
            (left, top, x_middle, y_middle),
            Some(index),
        );
        let ne_node = CNNode::<T, DefaultKey, S>::new(
            ne_item,
            parent_layer + 1,
            (x_middle, top, right, y_middle),
            Some(index),
        );
        let sw_node = CNNode::<T, DefaultKey, S>::new(
            sw_item,
            parent_layer + 1,
            (left, y_middle, x_middle, bottom),
            Some(index),
        );
        let se_node = CNNode::<T, DefaultKey, S>::new(
            se_item,
            parent_layer + 1,
            (x_middle, y_middle, right, bottom),
            Some(index),
        );

        // Insert child nodes.
        let nw_key = self.store.insert(nw_node);
        let ne_key = self.store.insert(ne_node);
        let sw_key = self.store.insert(sw_node);
        let se_key = self.store.insert(se_node);

        // Update child node neighbors.
        self.get_node_mut(nw_key).unwrap().update_neighbors([
            nw_w_neighbor,
            nw_n_neighbor,
            Some(ne_key),
            Some(sw_key),
        ]);
        self.store.get_mut(ne_key).unwrap().update_neighbors([
            Some(nw_key),
            ne_n_neighbor,
            ne_e_neighbor,
            Some(se_key),
        ]);
        self.store.get_mut(sw_key).unwrap().update_neighbors([
            sw_w_neighbor,
            Some(nw_key),
            Some(se_key),
            sw_s_neighbor,
        ]);
        self.store.get_mut(se_key).unwrap().update_neighbors([
            Some(sw_key),
            Some(ne_key),
            se_e_neighbor,
            se_s_neighbor,
        ]);

        // Update neighbor nodes to point to child nodes.
        self.update_neighbors_to_children(
            w_neighbors,
            nw_key,
            sw_key,
            sw_w_neighbor,
            Cardinality::West,
        );
        self.update_neighbors_to_children(
            n_neighbors,
            nw_key,
            ne_key,
            ne_n_neighbor,
            Cardinality::North,
        );
        self.update_neighbors_to_children(
            e_neighbors,
            se_key,
            ne_key,
            se_e_neighbor,
            Cardinality::East,
        );
        self.update_neighbors_to_children(
            s_neighbors,
            sw_key,
            se_key,
            sw_s_neighbor,
            Cardinality::South,
        );

        // Update parent.
        let parent = self.get_node_mut(index).unwrap();
        parent.update_neighbors([None, None, None, None]);
        parent.update_children(Some([nw_key, ne_key, sw_key, se_key]));

        if self.layers.len() <= parent_layer + 1 {
            self.layers.resize(parent_layer + 2, 0);
        }
        self.layers[parent_layer + 1] += 4;

        Ok([nw_key, ne_key, sw_key, se_key])
    }

    fn pop_children(&mut self, index: Self::Index) -> Option<[T; 4]> {
        let (parent_layer, children) = match self.get_node(index) {
            Some(n) if n.has_children() => (n.level(), n.get_children_index().unwrap()),
            _ => return None,
        };

        // Check if children has children.
        for child in children.iter() {
            if self.get_node(*child).unwrap().has_children() {
                return None;
            }
        }

        let [nw_key, ne_key, sw_key, se_key] = children;

        let w_cneighbor =
            self.get_and_update_children_neighbors(nw_key, sw_key, index, Cardinality::West);
        let n_cneighbor =
            self.get_and_update_children_neighbors(nw_key, ne_key, index, Cardinality::North);
        let e_cneighbor =
            self.get_and_update_children_neighbors(se_key, ne_key, index, Cardinality::East);
        let s_cneighbor =
            self.get_and_update_children_neighbors(se_key, sw_key, index, Cardinality::South);

        {
            let parent = self.get_node_mut(index).unwrap();
            parent.update_neighbors([w_cneighbor, n_cneighbor, e_cneighbor, s_cneighbor]);
            parent.update_children(None);
        }

        self.layers[parent_layer + 1] -= 4;

        let children = [
            self.store.remove(nw_key).unwrap().pop(),
            self.store.remove(ne_key).unwrap().pop(),
            self.store.remove(sw_key).unwrap().pop(),
            self.store.remove(se_key).unwrap().pop(),
        ];

        Some(children)
    }

    fn point_locate(
        &self,
        point: (
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
        ),
    ) -> Option<Self::Index> {
        let mut index = self.root_key;

        let mut node = self.get_node(index)?;
        if !node.point_in(point) {
            return None;
        }
        if !node.has_children() {
            return Some(index);
        }

        let (left, top, right, bottom) = node.get_bounds();
        let width: f32 = (right - left).to_f32().unwrap();
        let height: f32 = (bottom - top).to_f32().unwrap();

        // Converting point to [0, 1)x[0, 1) form
        let x: f32 = (point.0 - left).to_f32().unwrap() / width;
        let y: f32 = (point.1 - top).to_f32().unwrap() / height;

        let x_loc_code = (x * 2f32.powi(self.get_max_level() as i32)) as usize;
        let y_loc_code = (y * 2f32.powi(self.get_max_level() as i32)) as usize;

        // Current level = root level = max_level
        // So root's children's level is max_level - 1
        let mut next_level = self.get_max_level() - 1;

        while node.has_children() {
            let child_branch_bit = 1 << next_level;
            let child_index = ((x_loc_code & child_branch_bit) >> next_level)
                + ((y_loc_code & child_branch_bit) >> (next_level - 1));
            index = node.get_children_index().unwrap()[child_index];
            node = self.get_node(index).unwrap();
            next_level -= 1;
        }

        debug_assert!(node.point_in(point));

        Some(index)
    }

    fn region_locate(
        &self,
        region: (
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
            <Self::Node as RegionQuadtreeNode<T>>::Unit,
        ),
    ) -> Option<Vec<Self::Index>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_subdivide() {
        let mut tree = CNQuadtree::new("root".to_string(), (0, 0, 100, 100));
        let root = tree.get_root();

        let child_items = [
            "nw".to_string(),
            "ne".to_string(),
            "sw".to_string(),
            "se".to_string(),
        ];

        let children = tree.subdivide(root, child_items.clone());
        assert!(children.is_ok());
        for (i, child) in children.unwrap().into_iter().enumerate() {
            assert_eq!(tree.get_node(child).unwrap().get_item(), &child_items[i]);
        }
    }

    #[test]
    fn point_locate() {}
}
