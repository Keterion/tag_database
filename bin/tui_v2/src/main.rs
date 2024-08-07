use db::wrapper::Database;
fn main() {
    let db: Database = Database::open("base.db");
}
