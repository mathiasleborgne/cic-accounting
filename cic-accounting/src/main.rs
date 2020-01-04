extern crate csv;

use std::collections::HashMap;

#[derive(Debug)]
struct AccountingEntry {
    date_transaction: String,
    date_effect: String,
    amount: f32,
    label: String,
    category: ExpenseCategory,
}

#[derive(Debug)]
#[derive(PartialEq)]
enum ExpenseCategory {
    // strings might be better for this purpose
    Car,
    Transfer,
    Unknown,
}

fn get_category_from_label(label: &String) -> ExpenseCategory {
    if label.contains("VIR PEL") {
        return ExpenseCategory::Transfer
    } else if label.contains("APRR AUTOROUTE CARTE") {
        return ExpenseCategory::Car
    }
    return ExpenseCategory::Unknown
}

fn build_accounting_entry_from_cvs_record(record: &HashMap<String, String>) -> AccountingEntry {
    // todo: there might be a way to avoid cloning in here...
    AccountingEntry { 
        date_transaction: record["Date"].clone(), 
        date_effect: record["Datedevaleur"].clone(),
        amount: match record["Montant"].parse::<f32>() {
            Err(why) => panic!("{:?}", why),
            Ok(amount_float) => amount_float,
        }, 
        label: record["Libelle"].clone(),
        category: get_category_from_label(&record["Libelle"]),
    }
}

fn get_sum_all_amounts(accountings: &Vec<AccountingEntry>) -> f32 {
    let mut sum_all_amounts = 0.; 
    for accounting_entry in accountings {
        sum_all_amounts += accounting_entry.amount;
    }
    return sum_all_amounts
}

fn get_sum_category(accountings: &Vec<AccountingEntry>, category: ExpenseCategory) -> f32 {
    // todo: not so clean
    let mut sum_category = 0.;
    for accounting_entry in accountings {
        if accounting_entry.category == category {
            sum_category += accounting_entry.amount;
        }
    }
    return sum_category
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
    println!("----------------");
    println!("Car expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Car));
    println!("Transfer expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Transfer));
    println!("Unknown expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Unknown));
    println!("Unknown expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Unknown));
    println!("----------------");
    println!("Total expenses: {:?}", get_sum_all_amounts(&accountings));
    Ok(())
}

