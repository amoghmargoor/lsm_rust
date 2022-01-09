use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use rand::{thread_rng, Rng};

// SkipList Thread Safety
// ----------------------
// This implementation of SkipList is inspired by LevelDB implementation.
// Writes have to be synchronized between threads, but reads are lock free.
// Writes are appends only i.e., there is no delete or update supported.
// The memory for every node in the list is kept until SkipList is
// not destroyed.
// Caveats:
// 1. Same key cannot be inserted twice.
// 2. No update or deletes supported.
// 3. Writes need to be synchronized, but reads are lock free.
// TODO: Free up the memory

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
    fn new(key: Key) -> Box<Node<Key>> {
        Box::new(Node {
            key : key,
            nodes_: Default::default()
        })
    }

    fn head(dummy_key: Key) -> Box<Node<Key>> {
        Box::new(Node {
            key : dummy_key,
            nodes_: Default::default()
        })
    }

    #[inline(always)]
    fn set_next(&self, level: usize, node: &mut Node<Key>) {
        self.nodes_[level].store(node, Ordering::Release);
    }

    #[inline(always)]
    fn set_next_no_barrier(&self, level: usize, node: &mut Node<Key>) {
        self.nodes_[level].store(node, Ordering::Relaxed);
    }

    fn next(&self, level: usize) -> Option<&Node<Key>> {
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
    fn next_no_barrier(&self, level: usize) -> Option<&mut Node<Key>> {
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
    head_: Box<Node<Key>>,
    max_height_: AtomicUsize
}

impl<Key> SkipList<Key>
where Key: std::cmp::Ord {
    pub fn new(dummy_key: Key) -> SkipList<Key> {
        SkipList {
            head_ : Node::head(dummy_key),
            max_height_: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    fn equal(a: &Key, b: &Key) -> bool {
        a.cmp(b) == std::cmp::Ordering::Equal
    }

    pub fn insert(&self, key: Key) {
        let mut prev :[*const Node<Key>; MAX_HEIGHT] = [std::ptr::null(); MAX_HEIGHT];
        let node = self.find_greater_or_equal::<true>(&key, &mut prev);
        assert!(node.is_none() || !Self::equal(&node.unwrap().key, &key));
        let height = self.random_height();
        let max_height = self.max_height_.load(Ordering::Acquire);
        if height > max_height {
            for i in max_height+1..height+1 {
                prev[i] = &*self.head_;
            }
            self.max_height_.store(height, Ordering::Release);
        }
        let x = Box::into_raw(Box::new(Node::new(key)));
        for i in 0..height+1 {
            // Avoiding Barriers as we will apply Barrier
            // in the next statement.
            let next_node = unsafe {
                (*prev[i]).next_no_barrier(i)
            };         
            unsafe {
                if next_node.is_some() {
                    x.as_ref().unwrap().set_next_no_barrier(i, next_node.unwrap())
                };
                (*prev[i]).set_next(i, &mut *x);
            }
        }
    }

    fn find_greater_or_equal<const PREV: bool>(&self, key: &Key,
        prev: &mut [*const Node<Key>]) -> Option<&Node<Key>> {
        let mut x = &*self.head_;
        let mut level = self.get_max_level();
        loop {
            let next = x.next(level);
            if !next.is_none() && self.key_greater_than_node(key, next.unwrap()) {
                x = next.unwrap();
            } else {
                if PREV {
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
    fn key_greater_than_node(&self, key: &Key, x: &Node<Key>) -> bool {
        key.cmp(&x.key) == std::cmp::Ordering::Greater
    }

    pub fn contains(&self, key: &Key) -> bool {
        let node = self.find_greater_or_equal::<false>(key, Default::default());
        node.is_some() && Self::equal(key, &node.unwrap().key)
    }

    #[inline(always)]
    fn get_max_level(&self) -> usize {
        self.max_height_.load(Ordering::Relaxed)
    }

    fn random_height(&self) ->  usize {
        let mut height = 0;
        while height < MAX_HEIGHT - 1
            && (thread_rng().gen_range(0..BRANCHING) == 0) {
            height = height + 1;
        }
        return height;
    }
}

pub struct Iterator<'a, Key>
where Key: std::cmp::Ord {
    skiplist_: &'a SkipList<Key>,
    node_: Option<&'a Node<Key>>
}

impl<'a, Key> Iterator<'a, Key>
where Key: std::cmp::Ord {
    pub fn new(skiplist: &SkipList<Key>) -> Iterator<Key> {
        Iterator {
            skiplist_: skiplist,
            node_: skiplist.head_.next(0)
        }
    }

    #[inline(always)]
    pub fn has_next(&self) -> bool { self.node_.is_some() }

    #[inline(always)]
    pub fn next(&mut self) {
        assert!(self.has_next());
        self.node_ = self.node_.unwrap().next(0);
    }

    #[inline(always)]
    pub fn key(&self) -> &Key {
        assert!(self.has_next());
        return &self.node_.unwrap().key;
    }

    #[inline(always)]
    pub fn seek(&mut self, target : &Key) {
        self.node_ = self.skiplist_.find_greater_or_equal::<false>(
            target, Default::default());
    }

    #[inline(always)]
    pub fn seek_to_first(&mut self) {
        self.node_ = self.skiplist_.head_.next(0);
    }
}