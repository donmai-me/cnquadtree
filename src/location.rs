#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Debug)]
/// The four cardinal directions in the following order: West, North, East, and South.
pub enum Cardinality {
    West,
    North,
    East,
    South,
}

impl Cardinality {
    #[inline]
    /// Return the opposite direction of a cardinality.
    pub fn opposite(&self) -> Self {
        ((*self as usize + 2) % 4).try_into().unwrap()
    }

    #[inline]
    /// Return the succeeding neighbor direction as specified in the paper.
    /// See the algorithm in section 3.2
    pub fn next_neighbor(&self) -> Self {
        (3 - *self as usize).try_into().unwrap()
    }
}

impl TryFrom<usize> for Cardinality {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Cardinality::West),
            1 => Ok(Cardinality::North),
            2 => Ok(Cardinality::East),
            3 => Ok(Cardinality::South),
            _ => Err(format!("value should only be from 0 to 4. Given {}", value)),
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Debug)]
/// The location of a quadtree child node in relation to its siblings.
/// In the following order: NorthWest, NorthEast, SouthWest, SouthEast.
pub enum Location {
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl TryFrom<usize> for Location {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Location::NorthWest),
            1 => Ok(Location::NorthEast),
            2 => Ok(Location::SouthWest),
            3 => Ok(Location::SouthEast),
            _ => Err(format!("value should only be from 0 to 4. Given {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_opposite() {
        let directions = [
            Cardinality::West,
            Cardinality::North,
            Cardinality::East,
            Cardinality::South,
        ];
        let opposites = [
            Cardinality::East,
            Cardinality::South,
            Cardinality::West,
            Cardinality::North,
        ];

        for (dir, opp) in directions.into_iter().zip(opposites) {
            assert_eq!(dir.opposite(), opp);
        }
    }

    #[test]
    fn test_cardinality_succeeding_neighbor() {
        let directions = [
            Cardinality::West,
            Cardinality::North,
            Cardinality::East,
            Cardinality::South,
        ];
        let succeeding_neighbors = [
            Cardinality::South,
            Cardinality::East,
            Cardinality::North,
            Cardinality::West,
        ];

        for (dir, next_neighbor) in directions.into_iter().zip(succeeding_neighbors) {
            assert_eq!(dir.next_neighbor(), next_neighbor);
        }
    }
}
