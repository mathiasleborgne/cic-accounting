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
    category: ExpenseCategory,
}

#[derive(Debug)]
#[derive(PartialEq)]
enum ExpenseCategory {
    // todo: strings might be better for this purpose
    Car,
    Transfer,
    Unknown,
}

fn category_to_string(category: &ExpenseCategory) -> String {
    return match category {
        Car => "Car".to_string(),
        Transfer => "Transfer".to_string(),
        Unknown => "Unknown".to_string(),
    }
}

fn string_to_category(string_category: &String) -> ExpenseCategory {
    return match string_category.as_ref() {
        "Car" => ExpenseCategory::Car,
        "Transfer" => ExpenseCategory::Transfer,
        "Unknown" => ExpenseCategory::Unknown,
        _ => ExpenseCategory::Unknown,
    }
}

fn get_category_from_label(label: &String) -> ExpenseCategory {
    if label.contains("VIR PEL") {
        return ExpenseCategory::Transfer
    } else if label.contains("APRR AUTOROUTE CARTE") {
        return ExpenseCategory::Car
    }
    return ExpenseCategory::Unknown
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
        category: string_to_category(&record["Category"]),
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

fn write_csv_guessed_categories(accountings: &Vec<AccountingEntry>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path("account_guessed_categories.csv")?;
    wtr.write_record(&["Date", "Datedevaleur", "Montant", "Libelle", "Category"]);
    for accounting_entry in accountings {
        wtr.write_record(&[
            accounting_entry.date_transaction.clone(),
            accounting_entry.date_effect.clone(),
            accounting_entry.amount.to_string().clone(),
            accounting_entry.label.clone(),
            category_to_string(&accounting_entry.category), // todo: this always displays "Car"
            ])?;
        } 
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
    println!("Car expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Car));
    println!("Transfer expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Transfer));
    println!("Unknown expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Unknown));
    println!("Unknown expenses: {:?}", get_sum_category(&accountings, ExpenseCategory::Unknown));
    println!("----------------");
    println!("Total expenses: {:?}", get_sum_all_amounts(&accountings));
    println!("");
}

fn main() -> Result<(), csv::Error> {
    let accountings = read_csv("raw_account.csv".to_string(), &build_accounting_entry_from_raw_csv_record)?;
    print_accountings(&accountings);
    write_csv_guessed_categories(&accountings); 
        // todo: fix car everywhere
        // todo: guess from older CSVs
    let accountings_modified = read_csv("account_guessed_categories.csv".to_string(), &build_accounting_entry_from_csv_record_with_categories)?;
    print_accountings(&accountings_modified);
    Ok(())
}

