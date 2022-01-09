use skiplist;

[#test]
fn test_insert_onethread() {
    let s = SkipList<i32>::new();
    s.Insert(2);
    s.Insert(1);
    s.Insert(3);
    s.Insert(10);
    s.Insert(4);
    assert_eq!(s.Contains(1));
}

