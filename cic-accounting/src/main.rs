extern crate csv;

use std::collections::HashMap;

// https://www.reddit.com/r/rust/comments/bwplfl/read_csv_columns/
fn main() -> Result<(), csv::Error> {
    let mut rdr = csv::Reader::from_path("raw_account.csv")?;
    for result in rdr.deserialize() {
        let record: HashMap<String, String> = result?;
        println!(
            "column_1: {:?}, column_2: {:?}",
            record["column_1"],
            record["column_2"],
        );
    }
    Ok(())
}
