
extern crate csv;

use std::collections::HashMap;

#[derive(Debug)]
struct AccountingEntry {
    date_transaction: String,
    date_effect: String,
    amount: f32,
}

fn build_accounting_entry_from_cvs_record(record: &HashMap<String, String>) -> AccountingEntry {
    AccountingEntry { 
        date_transaction: record["column_1"].clone(), 
        date_effect: record["column_2"].clone(),
        amount: match record["unknown"].parse::<f32>() {
            Err(why) => panic!("{:?}", why),
            Ok(amount_float) => amount_float,
        }, 
    }
}

// https://www.reddit.com/r/rust/comments/bwplfl/read_csv_columns/
fn main() -> Result<(), csv::Error> {
    let mut rdr = csv::Reader::from_path("raw_account.csv")?;
    let mut accountings: Vec<AccountingEntry> = Vec::new();
    for result in rdr.deserialize() {
        let record: HashMap<String, String> = result?;
        println!(
            "column_1: {:?}, column_2: {:?}, unknown: {:?}",
            record["column_1"],
            record["column_2"],
            record["unknown"],
        );
        accountings.push(build_accounting_entry_from_cvs_record(&record));
    }
    println!("{:#?}", accountings);
    Ok(())
}

