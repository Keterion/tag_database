use crate::methods;
use rusqlite::Connection;

#[allow(private_interfaces)]

macro_rules! result_to_option {
    ($res:expr) => {
        match $res {
            Ok(val) => Some(val),
            Err(err) => {
                eprintln!("{} created error:\n{}", stringify!($expr), err);
                None
            }
        }
    };
}

pub struct Database {
    db: Connection,
}

impl Default for Database {
    fn default() -> Self {
        let db = Connection::open_in_memory().unwrap();
        Database {
            db: Connection::open_in_memory().unwrap(),
        }
    }
}

impl Database {
    /// Create a database with a connection to the given path
    pub fn open(path: &str) -> Self {
        Database {
            db: Connection::open(path).unwrap(),
        }
    }
    /// Create a database with a connection to the given path, also recreate the database fully
    pub fn create_db(path: &str) -> Self {
        Database {
            db: methods::init::recreate_db(path.into()),
        }
    }
}

/// Tag methods for the database
mod tags {
    use super::methods::{namespaces, tags};
    impl super::Database {
        /// Create tags with the names from the given Vec<&str>
        pub fn create_tags(&mut self, tags: Vec<&str>) {
            tags::add_tags(tags, &mut self.db);
        }
        /// Delete all tags via id given in the Vec<i64>
        pub fn delete_tags(&self, tags: Vec<i64>) -> Option<()> {
            result_to_option!(tags::remove_tags(tags, &self.db))
        }
        /// Get the name of the tag with the given id
        pub fn get_tag_name(&self, tag_id: i64) -> Option<String> {
            tags::get_name(tag_id, &self.db)
        }
        /// Rename the tag via id
        pub fn rename_tag(&self, tag_id: i64, new_name: &str) -> Option<()> {
            result_to_option!(tags::rename_tag(tag_id, new_name, &self.db))
        }
        /// Get all tag ids and names which don't connect to any images
        pub fn get_tag_orphans(&self) -> Vec<(i64, String)> {
            tags::get_orphans(&self.db)
        }
        /// Get all tags which match the given search term
        pub fn get_tags_with(&self, search_term: &str) -> Vec<(i64, String)> {
            tags::get_tags_with(search_term, &self.db)
        }
        /// Get the namespace of the tag, return None if it fails
        pub fn get_namespace_of_tag(&self, tag_id: i64) -> Option<i64> {
            namespaces::get_namespace_of_tag(tag_id, &self.db)
        }
    }
}

/// Image methods of the database
mod images {
    use super::methods::{images, tags};
    impl super::Database {
        /// Add a tag to the image, create decides if the tag should get created if it doesn't
        /// exist
        pub fn add_tag_to_img(&self, tag: &str, img_id: i64, create: bool) -> Option<()> {
            tags::add_tag_to_img(tag, img_id, create, &self.db)
        }
        /// Removes the tag matching the tag_id from the image matching the img_id
        pub fn remove_tag_from_img(&self, tag_id: i64, img_id: i64) -> Option<()> {
            result_to_option!(tags::remove_tag_from_img(tag_id, img_id, &self.db))
        }
        /// Creates an image entry with the given path and returns its id
        pub fn create_image(&self, path: &str) -> Option<i64> {
            images::add_image(path, &self.db)
        }
        /// Deletes the image with the path given
        pub fn delete_image(&self, path: &str) -> Option<()> {
            result_to_option!(images::remove_image_path(path, &self.db))
        }
        /// Gets all images with a given tag
        pub fn get_images_with_tag(&self, tag_id: i64) -> Option<Vec<(i64, String)>> {
            result_to_option!(images::get_images_with_tag(tag_id, &self.db))
        }
        /// Gets all tags of a given image
        pub fn get_tags_of_image(&self, img_id: i64) -> Vec<(i64, String)> {
            images::get_tags_of_img(img_id, &self.db)
        }
        /// Gets the path of an image via id
        pub fn get_image_path(&self, img_id: i64) -> Option<String> {
            images::get_path(img_id, &self.db)
        }
        /// Replaces the path of an image via id
        pub fn replace_image_path(&self, img_id: i64, path: &str) -> Option<()> {
            result_to_option!(images::update_path(img_id, path, &self.db))
        }
        /// Gets all images without tags associated to them
        pub fn get_image_orphans(&self) -> Vec<(i64, String)> {
            images::get_orphans(&self.db)
        }
        /// Gets all images matching a complex query
        pub fn complex_query(&self, query: &str) -> Vec<(i64, String)> {
            todo!()
        }
    }
}

/// Subtag methods of the database
mod subtags {
    use super::methods::subtags;
    impl super::Database {
        /// Gets all children and the children's children of a tag
        pub fn get_tag_children(&self, parent_id: i64) -> Option<Vec<i64>> {
            result_to_option!(subtags::get_children(parent_id, &self.db))
        }
        /// Gets all parents and parent's parents of a tag
        pub fn get_tag_parents(&self, child_id: i64) -> Option<Vec<i64>> {
            result_to_option!(subtags::get_parents(child_id, &self.db))
        }
        /// Adds a parent to a given tag, parent and tag need to exist
        pub fn add_tag_parent(&self, parent_id: i64, tag_id: i64) -> Option<()> {
            subtags::parent_tag(parent_id, tag_id, &self.db)
        }
        /// Removes the connection (parent-child / child-parent) from two tags
        pub fn remove_connection(&self, tag1: i64, tag2: i64) -> Option<()> {
            subtags::remove_connection(tag1, tag2, &self.db)
        }
    }
}

/// Namespace methods of the database
mod namespaces {
    use super::methods::namespaces;
    impl super::Database {
        /// Creates a namespace and returns its id
        pub fn create_namespace(&self, name: &str) -> Option<i64> {
            namespaces::add_namespace(name, &self.db)
        }
        /// Deletes a namespace via id
        pub fn delete_namespace(&self, namespace_id: i64) -> Option<()> {
            result_to_option!(namespaces::remove_namespace(namespace_id, &self.db))
        }
        /// Renames a namespace via id
        pub fn rename_namespace(&self, name: &str, namespace_id: i64) -> Option<()> {
            result_to_option!(namespaces::rename_namespace(namespace_id, name, &self.db))
        }
        /// Adds a namespace to a tag, each tag can only have one namespace
        pub fn add_namespace_to_tag(&self, namespace_id: i64, tag_id: i64) -> Option<()> {
            result_to_option!(namespaces::add_namespace_to_tag(
                namespace_id,
                tag_id,
                &self.db
            ))
        }
        /// Removes the namespace from a tag, each tag only has one namespace so no need to specify
        pub fn remove_namespace_from_tag(&self, tag_id: i64) -> Option<()> {
            result_to_option!(namespaces::remove_namespace_from_tag(tag_id, &self.db))
        }
        /// Gets tags which have the given namespace
        pub fn get_tags_with_namespace(&self, namespace_id: i64) -> Vec<(i64, String)> {
            namespaces::get_tags_with_namespace(namespace_id, &self.db)
        }
        /// Gets namespaces which don't link to any tags
        pub fn get_namespace_orphans(&self) -> Vec<(i64, String)> {
            namespaces::get_orphans(&self.db)
        }
    }
}
