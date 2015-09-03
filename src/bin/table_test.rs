extern crate uosql;

use uosql::storage::*;

fn main() {

    let t:Table = Default::default();
    println!("{:?}", t);
    let d = Database::new_database("bla");
    d.create_database();
    t.save("bla","MetaFile.meta");
    Table::load("bla","MetaFile.meta");
}
