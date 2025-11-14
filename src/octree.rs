use std::convert::TryInto;
use std::sync::Arc;

enum Child<T> {
    Node(Box<Node<T>>),
    Data(T),
}

struct Node<T> {
    children: [Option<Arc<Child<T>>>; 8],
}

pub struct Octree<T> {
    root: Box<Node<T>>,
    size: usize,
}

impl<T> Node<T> {
    fn new() -> Node<T> {
        Node {
            children: std::array::repeat(None),
        }
    }

    fn get(&self, x: isize, y: isize, z: isize, size: usize) -> Option<&T> {
    }
}

impl<T> Octree<T> {
    pub fn new(size: usize) -> Octree<T> {
        Octree {
            root: Box::new(Node::new()),
            size: size,
        }
    }

    fn is_valid_coordinate(&self, x: isize) -> bool {
        let size = self.size.try_into().unwrap();
        x < size && x >= -size
    }

    pub fn get(&self, x: isize, y: isize, z: isize) -> Option<&T> {
        if !self.is_valid_coordinate(x)
            || !self.is_valid_coordinate(y)
            || !self.is_valid_coordinate(z)
        {
            return None;
        }

        return None;
    }
}
