use itertools::Itertools;
use std::collections::HashSet;

/// Common trait for all UTnfa, Tnfa, Tdfa
pub trait Automata {
    /// Returns index of initial state
    fn begin(&self) -> usize;

    /// Returns number of nodes
    fn nodes(&self) -> usize;

    /// Returns `true` if `n` is a final state
    fn is_final(&self, n: usize) -> bool;

    /// Returns list of all non-epsilon edges
    fn list_edges(&self) -> impl Iterator<Item = (usize, usize, Option<u8>, isize)>;

    /// Returns `true` if `self` represents the same automata as `other`
    fn eq<T>(&self, other: &T) -> bool
    where
        T: Automata,
    {
        if self.nodes() != other.nodes() {
            return false;
        }
        let self_edges: HashSet<_> = HashSet::from_iter(self.list_edges());
        let other_edges: HashSet<_> = HashSet::from_iter(other.list_edges());
        if self_edges.len() != other_edges.len() {
            return false;
        }
        for v in (0..self.nodes()).permutations(self.nodes()) {
            if v[self.begin()] != other.begin() {
                continue;
            }
            for (i, n) in v.iter().enumerate() {
                if self.is_final(i) != other.is_final(*n) {
                    continue;
                }
            }
            if self_edges
                .iter()
                .all(|(a, b, c, t)| other_edges.contains(&(v[*a], v[*b], *c, *t)))
            {
                return true;
            }
        }
        return false;
    }
}

/// Generic implementation of Automata
#[derive(Debug)]
pub struct SimpleAutomata {
    pub begin: usize,
    pub nodes: usize,
    pub finals: HashSet<usize>,
    pub edges: Vec<(usize, usize, Option<u8>, isize)>,
}

impl Automata for SimpleAutomata {
    fn begin(&self) -> usize {
        self.begin
    }

    fn nodes(&self) -> usize {
        self.nodes
    }

    fn is_final(&self, n: usize) -> bool {
        self.finals.contains(&n)
    }

    fn list_edges(&self) -> impl Iterator<Item = (usize, usize, Option<u8>, isize)> {
        self.edges.iter().copied()
    }
}

impl<T: Automata> PartialEq<T> for SimpleAutomata {
    fn eq(&self, other: &T) -> bool {
        Automata::eq(self, other)
    }
}

#[cfg(test)]
mod automata_test {
    use super::*;

    #[test]
    fn simple_test() {
        let a = SimpleAutomata {
            begin: 0,
            nodes: 5,
            finals: HashSet::from([1, 2, 3]),
            edges: vec![
                (0, 1, Some(1), -1),
                (2, 1, Some(2), -1),
                (3, 1, Some(3), -1),
                (3, 4, Some(4), 2),
            ],
        };
        assert_eq!(a, a);

        let mut shifted = SimpleAutomata {
            begin: 1,
            nodes: 5,
            finals: HashSet::from([2, 3, 4]),
            edges: vec![
                (1, 2, Some(1), -1),
                (3, 2, Some(2), -1),
                (4, 2, Some(3), -1),
                (4, 0, Some(4), 2),
            ],
        };
        assert_eq!(a, shifted);
        assert_eq!(shifted, a);

        shifted.edges[2] = (3, 2, Some(3), -1);
        assert_ne!(a, shifted);
        assert_ne!(shifted, a);
    }
}
