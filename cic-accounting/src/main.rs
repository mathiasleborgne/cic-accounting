extern crate csv;

use std::collections::HashMap;

#[derive(Debug)]
struct AccountingEntry {
    date_transaction: String,
    date_effect: String,
    amount: f32,
    label: String,
}

fn build_accounting_entry_from_cvs_record(record: &HashMap<String, String>) -> AccountingEntry {
    // there might be a way to avoid cloning in here...
    AccountingEntry { 
        date_transaction: record["Date"].clone(), 
        date_effect: record["Datedevaleur"].clone(),
        amount: match record["Montant"].parse::<f32>() {
            Err(why) => panic!("{:?}", why),
            Ok(amount_float) => amount_float,
        }, 
        label: record["Libelle"].clone(),
    }
}

fn get_sum_all_amounts(accountings: Vec<AccountingEntry>) -> f32 {
    let mut sum_all_amounts = 0.; 
    for accounting_entry in accountings {
        sum_all_amounts += accounting_entry.amount;
    }
    return sum_all_amounts
}

// https://www.reddit.com/r/rust/comments/bwplfl/read_csv_columns/
fn main() -> Result<(), csv::Error> {
    let mut rdr = csv::Reader::from_path("raw_account.csv")?;
    let mut accountings: Vec<AccountingEntry> = Vec::new();
    for result in rdr.deserialize() {
        let record: HashMap<String, String> = result?;
        println!(
            "Date: {:?}, Datedevaleur: {:?}, Montant: {:?}",
            record["Date"],
            record["Datedevaleur"],
            record["Montant"],
        );
        accountings.push(build_accounting_entry_from_cvs_record(&record));
    }
    println!("{:#?}", accountings);
    println!("{:?}", get_sum_all_amounts(accountings));
    Ok(())
}

