extern crate uosql;

use uosql::storage::*;

fn main() {

    //let db = Database::new_database("storage_team").unwrap();
    let db = Database::load_database("storage_team").unwrap();

    let mut cols = Vec::new();
    cols.push(Column { name: "Heiner".into(), data_type: DataType::Integer });
    cols.push(Column { name: "Mathias".into(), data_type: DataType::Float });
    cols.push(Column { name: "Dennis".into(), data_type: DataType::Float });
    cols.push(Column { name: "Jana".into(), data_type: DataType::Integer });


    let _storage_team = db.create_table(1, cols, "storage_team").unwrap();

    let t = db.load_table("storage_team").unwrap();

    let mut engine = t.create_engine();
    let _e  = engine.create_table(t.columns());
}
