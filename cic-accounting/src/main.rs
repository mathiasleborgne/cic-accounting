extern crate csv;

use std::collections::HashMap;
use std::error::Error;
use csv::Writer;

#[derive(Debug)]
struct AccountingEntry {
    date_transaction: String,
    date_effect: String,
    amount: f32,
    label: String,
    category: String,
}




fn get_category_from_label(label: &String) -> String {
    if label.contains("VIR PEL") {
        return "Transfer".to_string()
    } else if label.contains("APRR AUTOROUTE CARTE") {
        return "Car".to_string()
    }
    return "Unknown".to_string()
}

fn build_accounting_entry_from_raw_csv_record(record: &HashMap<String, String>) -> AccountingEntry {
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

fn build_accounting_entry_from_csv_record_with_categories(record: &HashMap<String, String>) -> AccountingEntry {
    // todo: there might be a way to avoid cloning in here...
    AccountingEntry { 
        date_transaction: record["Date"].clone(), 
        date_effect: record["Datedevaleur"].clone(),
        amount: match record["Montant"].parse::<f32>() {
            Err(why) => panic!("{:?}", why),
            Ok(amount_float) => amount_float,
        }, 
        label: record["Libelle"].clone(),
        category: record["Category"].clone(),
    }
}

fn get_sum_all_amounts(accountings: &Vec<AccountingEntry>) -> f32 {
    let mut sum_all_amounts = 0.; 
    for accounting_entry in accountings {
        sum_all_amounts += accounting_entry.amount;
    }
    return sum_all_amounts
}

fn get_sum_category(accountings: &Vec<AccountingEntry>, category: String) -> f32 {
    // todo: not so clean
    let mut sum_category = 0.;
    for accounting_entry in accountings {
        if accounting_entry.category == category {
            sum_category += accounting_entry.amount;
        }
    }
    return sum_category
}

fn write_csv_guessed_categories(accountings: &Vec<AccountingEntry>) -> Result<(), Box<dyn Error>> {
    let path = "account_guessed_categories.csv";
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(&["Date", "Datedevaleur", "Montant", "Libelle", "Category"]);
    for accounting_entry in accountings {
        wtr.write_record(&[
            accounting_entry.date_transaction.clone(),
            accounting_entry.date_effect.clone(),
            accounting_entry.amount.to_string().clone(),
            accounting_entry.label.clone(),
            accounting_entry.category.clone(), // todo: this always displays "Car"
        ])?;
    } 
    println!("Saved file {:?}", path);
    Ok(())
}
    
fn read_csv(csv_path: String, entry_builder_function: &dyn Fn(&HashMap<String, String>)-> AccountingEntry) -> Result<Vec<AccountingEntry>, csv::Error> {
    // pass an entry_builder_function to read csv with or without categories
    // https://www.reddit.com/r/rust/comments/bwplfl/read_csv_columns/
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut accountings: Vec<AccountingEntry> = Vec::new();
    for result in rdr.deserialize() {
        let record: HashMap<String, String> = result?;
        accountings.push(entry_builder_function(&record));
    }
    return Ok(accountings)
}

fn print_accountings(accountings: &Vec<AccountingEntry>) {
    println!("{:#?}", accountings);
    println!("----------------");
    let all_expense_categories = [
        "Car",
        "Transfer",
        "Divers",
        "Unknown",
    ];
    for expense_category in all_expense_categories.iter() {        
        println!("{:?} expenses: {:?}", expense_category, get_sum_category(&accountings, expense_category.to_string()));
    }
    println!("----------------");
    println!("Total expenses: {:?}", get_sum_all_amounts(&accountings));
    println!("");
}

fn main() -> Result<(), csv::Error> {
    let accountings = read_csv("raw_account.csv".to_string(), &build_accounting_entry_from_raw_csv_record)?;
    print_accountings(&accountings);
    write_csv_guessed_categories(&accountings); 
        // todo: guess from older CSVs
        // todo: check dates
        // todo: split into 2 files for guessing, then summing
    println!("Modify {:?} and save it as {:?}", "account_guessed_categories".to_string(), "account_guessed_categories_modified".to_string());
    let accountings_modified = read_csv("account_guessed_categories_modified.csv".to_string(), &build_accounting_entry_from_csv_record_with_categories)?;
    print_accountings(&accountings_modified);
    Ok(())
}

