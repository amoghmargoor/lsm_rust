use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use rand::{thread_rng, Rng};
// Maximum height of the skiplist.
const MAX_HEIGHT: usize = 12;
// Used in Random generation of height for Skiplist nodes.
// Check the function 'RandomHeight'
const BRANCHING: usize = 5;
struct Node<Key> {
    key: Key;
    nodes_: &Vec<AtomicPtr<Node<Key>>>;
}

impl Node {
    fn new(key: Key, height: usize) -> Node {
        Node {
            key : Key,
            nodes_: vec![AtomicPtr, height]
        }
    }

    fn head() -> Node {
        Node {
            key : nullptr,
            nodes_: nullptr
        }
    }

    [#inline]
    fn SetNext(level: usize, key: &Node<Key>) {
        nodes_[level].store(AtomicPtr::new(key), Ordering::Release);
    }

    [#inline]
    fn SetNext_NoBarrier() {
        nodes_[level].store(AtomicPtr::new(key), Ordering::Relaxed);
    }

    [#inline]
    fn Next(level: usize) -> &Node<Key> {
        nodes_[level].get(Ordering::Acquire)
    }

    [#inline]
    fn Next_NoBarrier(level: usize) -> &Node<Key> {
        nodes_[level].get(Ordering::Relaxed)
    }
}

struct SkipList<Key ? Ord> {
    head_: Node<Key>,
    max_height_: AtomicUsize
}

impl SkipList<Key> {
    fn new() {
        SkipList {
            head_ : Node<Key>::head(),
            max_height_: AtomicUsize::new(0)
        }
    }
    fn Insert(key: Key) {
        let mut prev: Node[MAX_HEIGHT] = [nullptr; MAX_HEIGHT];
        let node = FindGreaterOrEqual<true>(key, &mut prev);
        assert(node == nullptr || !Equal(node.key, key))
        let height = RandomHeight();
        let max_height = max_height_.get(Ordering::Acquire)
        if (height > max_height) {
            for(let i = max_height; i <= MAX_HEIGHT; i++) {
                prev[i] = head_;
            }
            max_height_.store(Ordering::Release);
        }
        x = Node::new(key, height);
        for (let i = 0; i < height; i++) {
            // Avoiding Barriers as we will apply Barrier
            // in the next statement.
            x.SetNext_NoBarrier(i, prev[i].Next_NoBarrier(i));
            prev[i].SetNext(i, x);
        }
    }

    fn FindGreaterOrEqual<StorePrev: bool>(key: Key, prev: &mut Node<Key>[]) -> &Node<Key> {
        let x = head_;
        let level = GetMaxLevel() - 1;
        while (true) {
            let next = x.GetNext(level);
            if (KeyGreaterThanNode(key, next)) {
                x = next;
            } else {
                if (StorePrev) {
                    prev[level] = x;
                }
                if (level == 0) {
                    return next;
                }
                level--;
            }

        }
    }

    [#inline]
    fn Equal(a: &Key, b: &Key) -> bool { a.cmp(b) == std::cmp::Ordering::Equal }

    [#inline]
    fn KeyGreaterThanNode(key: Key, &x: Node<Key>) -> bool {
        x != nullptr && key.cmp(x.key) == std::cmp::Ordering::Greater
    }

    [#inline]
    fn Contains(key: Key) -> bool {
        let node = FindGreaterOrEqual<false>(key, nullptr);
        node != nullptr && Equal(key, node.key)
    }

    [#inline]
    fn GetMaxLevel() -> usize {
        max_height_.get(Ordering::Relaxed)
    }

    fn RandomHeight() ->  usize {
        let mut height = 1;
        while (height < MAX_HEIGHT
            && (thread_rng().gen_range(0..BRANCHING) == 0)) {
            height++;
        }
        assert(height > 0 && height <= MAX_HEIGHT);
        return height;
    }
}
#[cfg(test)]
mod skiplist_test;