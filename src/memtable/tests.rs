#[cfg(test)]
mod skiplist_test {
    use crate::memtable::skiplist::{Iterator, SkipList};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_empty_list() {
        let s = SkipList::<i32>::new(0);
        assert_eq!(s.contains(&0), false);
        let iter = Iterator::new(&s);
        assert_eq!(iter.has_next(), false);
    }

    #[test]
    fn test_insert_contains_onethread() {
        let s = SkipList::<i32>::new(Default::default());
        let mut expected_list = vec!();
        // Insert in order
        for i in 1..1000 {
            s.insert(i);
            expected_list.push(i);
        }
        let expected_len = expected_list.len(); 
        // Insert in reverse Order
        for i in (3000..4001).rev() {
            s.insert(i);
            expected_list.insert(expected_len, i);
        }
        // Search for existing keys
        for i in 1..1000 {
            assert!(s.contains(&i));
        }
        // Search for existing keys
        for i in 3000..4001 {
            assert!(s.contains(&i));
        }
        // Search for nonexisting keys
        for i in 1000..2000 {
            assert!(!s.contains(&i));
        }
        // Iterator tests
        let mut iter = Iterator::new(&s);
        for i in expected_list {
            assert!(iter.has_next());
            assert_eq!(iter.key(), &i);
            iter.next();
        }
        assert_eq!(iter.has_next(), false);
        iter.seek_to_first();
        assert_eq!(iter.key(), &1);
        // Seek to existing key
        iter.seek(&800);
        assert_eq!(iter.key(), &800);
        // Seek to non-existing key
        iter.seek(&5000);
        assert_eq!(iter.has_next(), false);
    }

    #[test]
    fn test_insert_contains_multithread() {
        let s = SkipList::new(0);
        let s_arc_mutex = Arc::new(Mutex::<()>::default());
        let s_arc_mutex_clone = s_arc_mutex.clone();
        // Insert in order
        rayon::scope(|thread_scope| {
            thread_scope.spawn(|_| {
                for i in 1..1000 {
                    let data = s_arc_mutex.lock().unwrap();
                    s.insert(i);
                    drop(data);
                }
            });
            // Insert in reverse Order
            for i in (3000..4001).rev() {
                let data = s_arc_mutex_clone.lock().unwrap();
                s.insert(i);
                drop(data);
            }
        });
        rayon::scope(|thread_scope| {
            thread_scope.spawn(|_| {
                for i in 1..1000 {
                    assert!(s.contains(&i));
                }
            });
            thread_scope.spawn(|_| {
                // Search for existing keys
                for i in 3000..4001 {
                    assert!(s.contains(&i));
                }
            });
            // Search for nonexisting keys
            for i in 1000..2000 {
                assert!(!s.contains(&i));
            }
        });
    }
}