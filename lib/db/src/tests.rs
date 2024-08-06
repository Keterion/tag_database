use crate::methods::*;
use rusqlite::Connection;

fn init_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    crate::methods::init::init_tables(&conn).unwrap();
    conn
}

#[cfg(test)]
mod util {
    use super::{init_db, tags, utils};

    #[test]
    fn remove_id() {
        let conn = init_db();
        let t_id = tags::add_tag("test", &conn).unwrap();
        utils::remove_id(t_id, "tags", &conn).unwrap();
        assert!(Option::is_none(&tags::get_name(t_id, &conn)));
    }
}

#[cfg(test)]
mod tag_tests {
    use super::{images, init_db, tags, utils};
    mod adding {
        use super::*;
        #[test]
        fn add_tag() {
            // does the same as get_tag_name because you need to check the insert worked
            // with getting the name and the other way around
            let conn = init_db();
            tags::add_tag("test", &conn).unwrap();
            assert_eq!(tags::get_name(1, &conn), Some("test".to_owned()));
        }
        #[test]
        fn add_tag_twice() {
            let conn = init_db();
            assert!(Option::is_some(&tags::add_tag("test", &conn)));
            assert!(Option::is_none(&tags::add_tag("test", &conn)));
        }
        #[test]
        fn add_multiple_tags() {
            let mut conn = init_db();
            let t_ids = tags::add_tags(vec!["test1", "test2", "test3"], &mut conn);
            assert_eq!(t_ids[0], Some(1));
            assert_eq!(t_ids[1], Some(2));
            assert_eq!(t_ids[2], Some(3));
        }
        #[test]
        fn add_multiple_tags_twice() {
            let mut conn = init_db();
            let t_ids = tags::add_tags(vec!["test", "test", "test"], &mut conn);
            assert_eq!(t_ids.len(), 3);
            assert_eq!(t_ids, vec![Some(1), None, None]);
        }
    }
    mod deleting {
        use super::*;
        #[test]
        fn delete_tag() {
            let conn = init_db();
            let t_id = tags::add_tag("test", &conn).unwrap();
            tags::remove_tag(t_id, &conn).unwrap();
            assert_eq!(None, utils::get_id("tags", "name='test'", &conn));
        }
        #[test]
        fn delete_nonexistent_tag() {
            let conn = init_db();
            tags::remove_tag(1, &conn).unwrap();
        }
    }
    mod getting {
        use super::*;
        #[test]
        fn get_tag_id() {
            let conn = init_db();
            tags::add_tag("test", &conn).unwrap();
            assert_eq!(Some(1), utils::get_id("tags", "name='test'", &conn));
        }
        #[test]
        fn get_tag_name() {
            let conn = init_db();
            tags::add_tag("test", &conn).unwrap();
            assert_eq!(tags::get_name(1, &conn), Some("test".to_owned()));
        }
        #[test]
        fn get_nonexistent_tag_name() {
            let conn = init_db();
            assert!(Option::is_none(&tags::get_name(1, &conn)));
        }

        #[test]
        fn get_orphaned_tags() {
            let conn = init_db();
            let t1 = tags::add_tag("tag1", &conn).unwrap();
            let t2 = tags::add_tag("tag2", &conn).unwrap();
            assert_eq!(
                tags::get_orphans(&conn),
                vec![(t1, "tag1".to_owned()), (t2, "tag2".to_owned())]
            );
            let im = images::add_image("test.jpg", &conn).unwrap();
            let _ = tags::add_tag_to_img("tag1", im, false, &conn);
            let _ = tags::add_tag_to_img("tag2", im, false, &conn);
            assert!(tags::get_orphans(&conn).is_empty());
        }
    }
    mod updating {
        use super::*;
        #[test]
        fn update_tag_name() {
            let conn = init_db();
            let t_id = tags::add_tag("abc", &conn).unwrap();
            assert_eq!(tags::get_name(t_id, &conn), Some("abc".to_owned()));
            tags::rename_tag(t_id, "test", &conn).unwrap();
            assert_eq!(tags::get_name(t_id, &conn), Some("test".to_owned()));
        }
    }
}

#[cfg(test)]
mod image_test {
    use super::{images, init_db, tags, utils};
    mod adding {
        use super::*;
        #[test]
        fn add_img() {
            let conn = init_db();
            images::add_image("test.jpg", &conn).unwrap();
            assert_eq!(
                utils::get_id("images", "path='test.jpg'", &conn),
                Some(1)
            );
        }
        #[test]
        fn add_img_twice() {
            let conn = init_db();
            assert_eq!(Some(1), images::add_image("test.jpg", &conn));
            assert!(Option::is_none(&images::add_image("test.jpg", &conn)));
        }
        #[test]
        fn add_tag_to_img() {
            let conn = init_db();
            let _ = tags::add_tag("test", &conn);
            let im_id = images::add_image("test.jpg", &conn).unwrap();
            tags::add_tag_to_img("test", im_id, true, &conn).unwrap();
            assert_eq!(
                images::get_tags_of_img(im_id, &conn),
                vec![(1, "test".to_owned())]
            );
        }
        #[test]
        #[should_panic]
        fn add_unknown_tag_to_img() {
            let conn = init_db();
            let im_id = images::add_image("test.jpg", &conn).unwrap();
            tags::add_tag_to_img("test", im_id, false, &conn).unwrap();
        }
        #[test]
        fn add_tag_to_img_twice() {
            let conn = init_db();
            let _ = tags::add_tag("test", &conn);
            let im_id = images::add_image("test.jpg", &conn).unwrap();
            tags::add_tag_to_img("test", im_id, true, &conn).unwrap();
            tags::add_tag_to_img("test", im_id, true, &conn).unwrap();
        }
    }
    mod deleting {
        use super::*;
        #[test]
        fn delete_image() {
            let conn = init_db();
            let _ = images::add_image("test.jpg", &conn);
            images::remove_image_path("test.jpg", &conn).unwrap();
            assert_eq!(
                utils::get_id("images", "path='test.jpg'", &conn),
                None
            );
        }
    }
    mod getting {
        use super::*;
        #[test]
        fn get_tags_of_nonexistent_image() {
            let conn = init_db();
            assert!(images::get_tags_of_img(1, &conn).is_empty());
        }

        #[test]
        fn get_tags_of_image() {
            let conn = init_db();
            let im_id = images::add_image("test.jpg", &conn).unwrap();
            let _ = tags::add_tag_to_img("test", im_id, true, &conn);
            assert_eq!(
                images::get_tags_of_img(1, &conn),
                vec![(1, "test".to_owned())]
            );
        }

        #[test]
        fn get_path() {
            let conn = init_db();
            let im_id = images::add_image("test.jpg", &conn).unwrap();
            assert_eq!(images::get_path(im_id, &conn), Some("test.jpg".to_owned()));
            let _ = utils::remove_id(im_id, "images", &conn);
            assert_eq!(images::get_path(im_id, &conn), None);
        }
        #[test]
        fn get_orphans() {
            let conn = init_db();
            let im1 = images::add_image("test1.jpg", &conn).unwrap();
            let im2 = images::add_image("test2.jpg", &conn).unwrap();
            assert_eq!(
                images::get_orphans(&conn),
                vec![(im1, "test1.jpg".to_owned()), (im2, "test2.jpg".to_owned())]
            );
            let _ = tags::add_tag_to_img("test", im1, true, &conn);
            assert_eq!(
                images::get_orphans(&conn),
                vec![(im2, "test2.jpg".to_owned())]
            );
            let _ = tags::add_tag_to_img("test", im2, false, &conn);
            assert!(images::get_orphans(&conn).is_empty());
        }
    }
}

#[cfg(test)]
mod namespaces_and_parents {
    use super::{init_db, namespaces, subtags, tags, utils};

    mod namespacing {
        use super::{init_db, namespaces, tags, utils};

        mod adding {
            use super::*;

            #[test]
            fn add_namespace() {
                let conn = init_db();
                let ns_id = namespaces::add_namespace("test", &conn).unwrap();
                assert_eq!(
                    utils::get_id("namespaces", "name='test'", &conn),
                    Some(ns_id)
                );
            }
            #[test]
            fn remove_namespace() {
                let conn = init_db();
                let ns_id = namespaces::add_namespace("test", &conn).unwrap();
                utils::remove_id(ns_id, "namespaces", &conn).unwrap();
                assert!(Option::is_none(&utils::get_id(
                    "namespaces",
                    "name='test'",
                    &conn
                )));
            }

            #[test]
            fn add_namespace_to_tag() {
                let conn = init_db();
                let ns_id = namespaces::add_namespace("test", &conn).unwrap();
                let t_id = tags::add_tag("testtag", &conn).unwrap();
                namespaces::add_namespace_to_tag(ns_id, t_id, &conn).unwrap();
                assert_eq!(namespaces::get_namespace_of_tag(t_id, &conn), Some(ns_id));
                let _ = utils::remove_id(ns_id, "namespaces", &conn);
                assert_eq!(namespaces::get_namespace_of_tag(t_id, &conn), None);
                assert!(Option::is_none(&namespaces::get_namespace_name(
                    ns_id, &conn
                )));
            }
        }
        mod deleting {
            use super::*;

            #[test]
            fn remove_namespace_from_tag() {
                let conn = init_db();
                let t_id = tags::add_tag("testtag", &conn).unwrap();
                let ns_id = namespaces::add_namespace("test", &conn).unwrap();
                namespaces::add_namespace_to_tag(ns_id, t_id, &conn).unwrap();
                assert_eq!(namespaces::get_namespace_of_tag(t_id, &conn), Some(1));
                namespaces::remove_namespace_from_tag(t_id, &conn).unwrap();
                assert_eq!(namespaces::get_namespace_of_tag(t_id, &conn), None);
            }
            #[test]
            fn remove_namespace() {
                let conn = init_db();
                let ns_id = namespaces::add_namespace("ns1", &conn).unwrap();
                assert_eq!(
                    namespaces::get_namespace_name(ns_id, &conn),
                    Some("ns1".to_owned())
                );
                namespaces::remove_namespace(ns_id, &conn).unwrap();
                assert!(Option::is_none(&utils::get_id(
                    "namespaces",
                    "name='ns1'",
                    &conn
                )));
            }
        }
        mod getting {
            use super::*;

            #[test]
            fn get_namespace_name() {
                let conn = init_db();
                let nid = namespaces::add_namespace("test", &conn).unwrap();
                assert_eq!(
                    namespaces::get_namespace_name(nid, &conn),
                    Some("test".to_owned())
                );
            }
            #[test]
            fn get_tags_with_namespace() {
                let conn = init_db();
                let t1 = tags::add_tag("tag1", &conn).unwrap();
                let t2 = tags::add_tag("tag2", &conn).unwrap();
                let ns = namespaces::add_namespace("test", &conn).unwrap();
                let _ = namespaces::add_namespace_to_tag(ns, t1, &conn);
                let _ = namespaces::add_namespace_to_tag(ns, t2, &conn);
                assert_eq!(
                    namespaces::get_tags_with_namespace(ns, &conn),
                    vec![(t1, "tag1".to_owned()), (t2, "tag2".to_owned()),]
                );
            }
            #[test]
            fn get_orphans() {
                let conn = init_db();
                let ns1 = namespaces::add_namespace("ns1", &conn).unwrap();
                let ns2 = namespaces::add_namespace("ns2", &conn).unwrap();
                assert_eq!(
                    namespaces::get_orphans(&conn),
                    vec![(ns1, "ns1".to_owned()), (ns2, "ns2".to_owned())]
                );
                let t_id = tags::add_tag("test", &conn).unwrap();
                namespaces::add_namespace_to_tag(ns1, t_id, &conn).unwrap();
                assert_eq!(
                    namespaces::get_orphans(&conn),
                    vec![(ns2, "ns2".to_owned())]
                );
                namespaces::remove_namespace_from_tag(t_id, &conn).unwrap();
                namespaces::add_namespace_to_tag(ns2, t_id, &conn).unwrap();
                assert_eq!(
                    namespaces::get_orphans(&conn),
                    vec![(ns1, "ns1".to_owned())]
                );
                namespaces::remove_namespace(ns1, &conn).unwrap();
                namespaces::remove_namespace(ns2, &conn).unwrap();
                assert!(namespaces::get_orphans(&conn).is_empty());
            }
        }
        mod updating {
            use super::*;
            #[test]
            fn update_namespace_name() {
                let conn = init_db();
                let ns_id = namespaces::add_namespace("testspace", &conn).unwrap();
                assert_eq!(
                    namespaces::get_namespace_name(ns_id, &conn).unwrap(),
                    "testspace"
                );
            }
        }
    }
    mod parents {
        use super::{init_db, subtags, tags, utils};
        mod adding {
            use super::*;

            #[test]
            fn parent_tag() {
                let mut conn = init_db();
                let t_ids = tags::add_tags(vec!["one", "two"], &mut conn);
                subtags::parent_tag(t_ids[0].unwrap(), t_ids[1].unwrap(), &conn).unwrap();
                assert_eq!(
                    utils::get_id("subtag_map", "parent_id=1 AND child_id=2", &conn),
                    Some(1)
                );
            }
            #[test]
            #[should_panic]
            fn create_loop() {
                let mut conn = init_db();
                let t_ids = tags::add_tags(vec!["one", "two"], &mut conn);
                let _ = subtags::parent_tag(t_ids[0].unwrap(), t_ids[1].unwrap(), &conn);
                subtags::parent_tag(t_ids[1].unwrap(), t_ids[0].unwrap(), &conn).unwrap();
                subtags::get_parents(1, &conn).unwrap();
            }
        }
        mod deleting {
            use super::*;

            #[test]
            fn delete_connection() {
                let mut conn = init_db();
                let t_ids = tags::add_tags(vec!["a", "b"], &mut conn);
                let _ = subtags::parent_tag(t_ids[0].unwrap(), t_ids[1].unwrap(), &conn);
                subtags::remove_connection("a", "b", &conn).unwrap();
                assert_eq!(utils::get_id("tags", "name='a'", &conn), t_ids[0]);
                assert_eq!(utils::get_id("tags", "name='b'", &conn), t_ids[1]);
                assert!(subtags::get_children(t_ids[0].unwrap(), &conn)
                    .unwrap()
                    .is_empty());
                assert!(subtags::get_parents(t_ids[1].unwrap(), &conn)
                    .unwrap()
                    .is_empty());
            }
            #[test]
            fn delete_connection_twice() {
                let mut conn = init_db();
                let t_ids = tags::add_tags(vec!["a", "b"], &mut conn);
                let _ = subtags::parent_tag(t_ids[0].unwrap(), t_ids[1].unwrap(), &conn);
                subtags::remove_connection("a", "b", &conn).unwrap();
                subtags::remove_connection("a", "b", &conn).unwrap();
            }
        }
        mod getting {
            use super::*;

            #[test]
            fn get_parent() {
                let mut conn = init_db();
                let t_ids = tags::add_tags(vec!["one", "two"], &mut conn);
                let _ = subtags::parent_tag(t_ids[0].unwrap(), t_ids[1].unwrap(), &conn);
                let mut parents = subtags::get_parents(t_ids[1].unwrap(), &conn).unwrap();
                assert_eq!(parents, vec![t_ids[0].unwrap()]);
                let three = tags::add_tag("three", &conn).unwrap();
                let _ = subtags::parent_tag(t_ids[1].unwrap(), three, &conn);
                parents = subtags::get_parents(three, &conn).unwrap();
                assert_eq!(parents, vec![t_ids[1].unwrap(), t_ids[0].unwrap()]);
            }
            #[test]
            fn get_child() {
                let mut conn = init_db();
                let t_ids = tags::add_tags(vec!["one", "two", "three"], &mut conn);
                let _ = subtags::parent_tag(t_ids[0].unwrap(), t_ids[1].unwrap(), &conn);
                let _ = subtags::parent_tag(t_ids[1].unwrap(), t_ids[2].unwrap(), &conn);
                let mut children = subtags::get_children(t_ids[1].unwrap(), &conn).unwrap();
                assert_eq!(children, vec![t_ids[2].unwrap()]);
                children = subtags::get_children(t_ids[0].unwrap(), &conn).unwrap();
                assert_eq!(children, vec![t_ids[1].unwrap(), t_ids[2].unwrap()]);
            }
            #[test]
            fn get_none() {
                let conn = init_db();
                let _ = tags::add_tag("a", &conn);
                assert!(subtags::get_parents(1, &conn).unwrap().is_empty());
                assert!(subtags::get_children(1, &conn).unwrap().is_empty());
            }
        }
    }
}
