extern crate uosql;

use uosql::storage::*;

fn main() {
    let t = Table::create_new();
    println!("{:?}", t);
    t.save("bla","MetaFile.meta");
}
