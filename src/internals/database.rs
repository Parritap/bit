use std::collections::HashMap;

/// We need a file a place to persist the database.

// HashMap that stores binary objects based their hash.
struct object_db {
    map: HashMap<String, Vec<u8>>,
}
