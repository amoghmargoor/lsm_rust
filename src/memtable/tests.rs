#[cfg(test)]
mod skiplist_test {
    use crate::memtable::skiplist;
    #[test]
    fn test_insert_onethread() {
        let s = crate::memtable::skiplist::SkipList::<i32>::new(0);
        s.Insert(2);
        s.Insert(1);
        s.Insert(3);
        s.Insert(10);
        s.Insert(4);
        let key = 1;
        assert!(s.Contains(&key));
    }
}

