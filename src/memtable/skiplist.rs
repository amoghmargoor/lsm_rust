use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use rand::{thread_rng, Rng};
// Maximum height of the skiplist.
const MAX_HEIGHT: usize = 12;
// Used in Random generation of height for Skiplist nodes.
// Check the function 'RandomHeight'
const BRANCHING: usize = 5;

struct Node<Key> {
    key: Key,
    nodes_: [AtomicPtr<Node<Key>>; MAX_HEIGHT],
}

impl<Key> Node<Key> {
    fn new(key: Key) -> Node<Key> {
        Node {
            key : key,
            nodes_: Default::default()
        }
    }

    fn head(dummyKey: Key) -> Node<Key> {
        Node {
            key : dummyKey,
            nodes_: Default::default()
        }
    }

    #[inline(always)]
    fn SetNext(&self, level: usize, key: &mut Node<Key>) {
        self.nodes_[level].store(key, Ordering::Release);
    }

    #[inline(always)]
    fn SetNext_NoBarrier(&self, level: usize, key: &mut Node<Key>) {
        self.nodes_[level].store(key, Ordering::Relaxed);
    }

    #[inline(always)]
    fn Next(&self, level: usize) -> Option<&Node<Key>> {
        let node = self.nodes_[level].load(Ordering::Acquire);
        if node.is_null() {
            return Option::None;
        } else {
            unsafe {
                return Option::from(&*node);
            }
        }
    }

    #[inline(always)]
    fn Next_NoBarrier(&self, level: usize) -> Option<&mut Node<Key>> {
        let node = self.nodes_[level].load(Ordering::Relaxed);
        if node.is_null() {
            return Option::None;
        } else {
            unsafe {
                return Option::from(&mut *node);
            }
        }
    }
}

pub struct SkipList<Key>
where Key: std::cmp::Ord {
    head_: Node<Key>,
    max_height_: AtomicUsize
}

impl<Key> SkipList<Key>
where Key: std::cmp::Ord {
    pub fn new(dummyKey: Key) -> SkipList<Key> {
        SkipList {
            head_ : Node::head(dummyKey),
            max_height_: AtomicUsize::new(0)
        }
    }
    #[inline(always)]
    fn Equal(a: &Key, b: &Key) -> bool { a.cmp(b) == std::cmp::Ordering::Equal }

    pub fn Insert(&self, key: Key) {
        let mut prev :[*const Node<Key>; MAX_HEIGHT] = [std::ptr::null(); MAX_HEIGHT];
        let node = self.FindGreaterOrEqual::<true>(&key, &mut prev);
        assert!(node.is_none() || !Self::Equal(&node.unwrap().key, &key));
        let height = self.RandomHeight();
        let max_height = self.max_height_.load(Ordering::Acquire);
        if height > max_height {
            for i in max_height..MAX_HEIGHT+1 {
                prev[i] = &self.head_;
            }
            self.max_height_.store(height, Ordering::Release);
        }
        let mut x = Node::new(key);
        for i in 0..height {
            // Avoiding Barriers as we will apply Barrier
            // in the next statement.
            let next_node = unsafe { (*prev[i]).Next_NoBarrier(i).unwrap() };
            x.SetNext_NoBarrier(i, next_node);
            unsafe {
                (*prev[i]).SetNext(i, &mut x);
            }
        }
    }

    fn FindGreaterOrEqual<const StorePrev: bool>(&self, key: &Key, prev: &mut [*const Node<Key>]) -> Option<&Node<Key>> {
        let mut x = &self.head_;
        let mut level = self.GetMaxLevel() - 1;
        loop {
            let next = x.Next(level);
            if !next.is_none() && self.KeyGreaterThanNode(key, next.unwrap()) {
                x = next.unwrap();
            } else {
                if StorePrev {
                    prev[level] = x;
                }
                if level == 0 {
                    return next;
                }
                level -= 1;
            }
        }
    }

    #[inline(always)]
    fn KeyGreaterThanNode(&self, key: &Key, x: &Node<Key>) -> bool {
        key.cmp(&x.key) == std::cmp::Ordering::Greater
    }

    #[inline(always)]
    pub fn Contains(&self, key: &Key) -> bool {
        let node = self.FindGreaterOrEqual::<false>(key, Default::default());
        node.is_some() && Self::Equal(key, &node.unwrap().key)
    }

    #[inline(always)]
    fn GetMaxLevel(&self) -> usize {
        self.max_height_.load(Ordering::Relaxed)
    }

    fn RandomHeight(&self) ->  usize {
        let mut height = 1;
        while height < MAX_HEIGHT
            && (thread_rng().gen_range(0..BRANCHING) == 0) {
            height = height +1;
        }
        assert!(height > 0 && height <= MAX_HEIGHT);
        return height;
    }
}