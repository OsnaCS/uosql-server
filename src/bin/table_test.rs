extern crate uosql;

use uosql::storage::*;

fn main() {

    let t = Table::create_new();
    println!("{:?}", t);
    let d = Database::new_database("bla");
    d.create_database();
    t.save("bla","MetaFile.meta");
    Table::load("bla","MetaFile.meta");
}
