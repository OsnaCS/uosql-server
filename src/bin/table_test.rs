extern crate uosql;

use uosql::storage::*;

fn main() {

    let t = Table::create_new();
    println!("{:?}", t);
    Database::create_database("bla");
    t.save("bla","MetaFile.meta");
    Table::load("bla","MetaFile.meta");
}
