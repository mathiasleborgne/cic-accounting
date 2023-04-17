// Source for CIC accounting
// 2 main parts:
//     - guess: parses a csv file and tries to guess the category of spending of each entry (based on previous categories inputs)
//     - sum: after user manually fills the unguessed categories, the sums are calculated, and categories are saved for later

extern crate csv;
extern crate chrono;
extern crate clap;

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::fs::File;
use csv::Writer;
use std::io::{self, Read, Write};
use crate::chrono::Datelike;
use regex::Regex;
use clap::{Arg, App, SubCommand};


const ALL_EXPENSE_CATEGORIES: [&str; 18] = [
    "Salaire",
    "Loyer",
    "Courses",
    "RE",
    "Fixes",
    "Divers",
    "Restaurants",
    "Voiture",
    "RATP",
    "SNCF",
    "Retraits",
    "RetraitsSO",
    "RetraitsP",
    "Impots",
    "DepensesSpe",
    "GainsSpe",
    "VirComptes",
    "Unknown",
];
const PATH_MODIFIED_ACCOUNTS: &'static str = "./modified_accounts/";

#[derive(Debug)]
struct AccountingEntry {
    date_transaction: chrono::NaiveDate,
    date_effect: String,
    amount: f32,
    label: String,
    category: String,
}

fn get_category_from_label(label: &String, known_labels_categories_map: &HashMap<String, String>) 
    -> String {
    let mut guessed_category = match known_labels_categories_map.get(&get_label_without_number(label)) {
        Some(category) => category.clone(),
        _ => "Unknown".to_string(),
    };
    if guessed_category == "RetraitsSO" || guessed_category == "RetraitsP" {
        guessed_category = "Retraits".to_string();
    }
    guessed_category
}

fn build_accounting_entry_from_raw_csv_record(
    record: &HashMap<String, String>, known_labels_categories_map: &HashMap<String, String>) 
    -> AccountingEntry {
    // todo: there might be a way to avoid cloning in here...
    AccountingEntry { 
        date_transaction: get_date_from_hashmap_record(record), 
        date_effect: record["Datedevaleur"].clone(),
        amount: match record["Montant"].parse::<f32>() {
            Err(why) => panic!("{:?}", why),
            Ok(amount_float) => amount_float,
        }, 
        label: record["Libelle"].clone(),
        category: get_category_from_label(&record["Libelle"], &known_labels_categories_map),
    }
}

fn get_date_from_hashmap_record(record: &HashMap<String, String>) -> chrono::NaiveDate {
    match chrono::NaiveDate::parse_from_str(&record["Date"].to_string(), "%m/%d/%Y") {
        Err(why) => panic!("Wrong date: {:?}", why),
        Ok(date_ok) => date_ok,
    }
}

fn build_accounting_entry_from_csv_record_with_categories(record: &HashMap<String, String>) 
    -> AccountingEntry {
    // todo: there might be a way to avoid cloning in here...
    AccountingEntry { 
        date_transaction: get_date_from_hashmap_record(&record), 
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

fn get_sum_category(accountings: &Vec<AccountingEntry>, category: &str) -> f32 {
    // todo: not so clean
    let mut sum_category = 0.;
    for accounting_entry in accountings {
        if accounting_entry.category == category {
            sum_category += accounting_entry.amount;
        }
    }
    return sum_category
}

fn write_csv_guessed_categories(accountings: &Vec<AccountingEntry>, file_name_guessed: &str) 
    -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(file_name_guessed)?;
    wtr.write_record(&["Date", "Datedevaleur", "Montant", "Libelle", "Category"])?;
    for accounting_entry in accountings {
        wtr.write_record(&[
            accounting_entry.date_transaction.format("%m/%d/%Y").to_string(),
            accounting_entry.date_effect.clone(),
            accounting_entry.amount.to_string().clone(),
            accounting_entry.label.clone(),
            accounting_entry.category.clone(), // todo: this always displays "Voiture"
        ])?;
    } 
    println!("Saved file {:?}", file_name_guessed);
    Ok(())
}

fn is_date_transaction_in_month_year(date_transaction: chrono::NaiveDate, month: u32, year: i32) 
    -> bool {
    let date_second = chrono::NaiveDate::from_ymd(year, month, 2);
    // the first day of the next month
    let (year_next_month, month_next) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let date_first_next = chrono::NaiveDate::from_ymd(year_next_month, month_next, 1);

    date_transaction >= date_second && date_transaction <= date_first_next
}
    
fn read_csv(csv_path: &str, month_year: Option<(u32, i32)>, 
            entry_builder_function: &dyn Fn(&HashMap<String, String>) -> AccountingEntry) 
    -> Result<Vec<AccountingEntry>, csv::Error> {
    // pass an entry_builder_function to read csv with or without categories
    // https://www.reddit.com/r/rust/comments/bwplfl/read_csv_columns/
    let mut reader = csv::Reader::from_path(csv_path)?;
    let mut accountings: Vec<AccountingEntry> = Vec::new();
    for result in reader.deserialize() {
        let record: HashMap<String, String> = result?;
        let accounting_entry = entry_builder_function(&record);
        match month_year {
            None => accountings.push(accounting_entry),
            Some((month, year)) => {
                if is_date_transaction_in_month_year(accounting_entry.date_transaction, month, year) {
                    accountings.push(accounting_entry);
                }
            },
        }
    }
    return Ok(accountings)
}

fn read_balance_from_csv(csv_path: &str, month: u32, year: i32) 
    -> Result<f32, csv::Error> {
    let mut reader = csv::Reader::from_path(csv_path)?;
    let mut balance: Option<f32> = None;
    for result in reader.deserialize() {
        let record: HashMap<String, String> = result?;
        let date = get_date_from_hashmap_record(&record);
        let date_first = chrono::NaiveDate::from_ymd(year, month, 1);
        let record_balance = match record["Solde"].parse::<f32>() {
            Ok(balance) => balance,
            Err(why) => panic!("balance should be a number: {:?}", why),
        };
        if date == date_first {
            balance = Some(record_balance)
        }
        else if date >= date_first {
            match balance {
                None => {
                    balance = Some(record_balance)
                },
                Some(_) => {},
            }
        }
    }
    return Ok(balance.unwrap())
}

fn check_categories(accountings: &Vec<AccountingEntry>) {
    for accounting_entry in accountings {
        match ALL_EXPENSE_CATEGORIES.iter().find(| &&x| x == accounting_entry.category) {
            Some(_) => {},
            None => panic!("Category not found: {:?}", accounting_entry.category),
        }
    }
}


fn print_category(accountings: &Vec<AccountingEntry>, category: &str) {
    let sum_category = get_sum_category(&accountings, category);
    println!("- {:?}: {:?}", category, sum_category);
    for accounting_entry in accountings {
        if accounting_entry.category == category {
            println!("    * {:?}: {:?}", accounting_entry.amount, accounting_entry.label);
        }
    }
}

fn print_accountings(accountings: &Vec<AccountingEntry>, current_month: Option<u32>) {
    // println!("{:#?}", accountings);
    let month = match current_month {
        Some(month) => month,
        None => accountings[0].date_transaction.month(),
    };
    println!("Sum for {:?} accounting entries for month {:?}", accountings.len(), month);
    println!("----------------");
    for expense_category in ALL_EXPENSE_CATEGORIES.iter() {        
        println!("{:?} expenses: {:?}", 
                 expense_category, get_sum_category(&accountings, expense_category));
    }
    println!("----------------");
    println!("Total balance: {:?}", get_sum_all_amounts(&accountings));
    println!("");
    println!("Special categories:");
    print_category(&accountings, "Divers");
    print_category(&accountings, "DepensesSpe");
    print_category(&accountings, "GainsSpe");
}

fn get_label_without_number(label: &String) -> String {
    let re = Regex::new(r"[0-9]").unwrap();
    return re.replace_all(label, "").to_string();
}

fn get_known_labels_categories_map() -> Result<HashMap<String, String>, csv::Error> {
    let paths = fs::read_dir(PATH_MODIFIED_ACCOUNTS).unwrap();
    let mut known_labels_categories_map = HashMap::new();
    for path in paths {
        // println!("Name: {}", path.unwrap().path().display());
        match path.unwrap().path().to_str() {
            None => panic!("new path is not a valid UTF-8 sequence"),
            Some(path_str) => {
                let accountings_guessed_file = 
                    read_csv(&path_str.to_string(), None, 
                             &build_accounting_entry_from_csv_record_with_categories)?;
                for accounting_guessed in accountings_guessed_file {
                    known_labels_categories_map.insert(
                        get_label_without_number(&accounting_guessed.label.clone()) , 
                                                 accounting_guessed.category.clone());
                }
            },
        }
    }
    // println!("Labels map: {:?}", known_labels_categories_map);
    // example: 
    // known_labels_categories_map.insert(get_label_without_number(&"PAIEMENT CB  CHAVILLE MONOPRIX CARTE ".to_string()), "Courses".to_string());
    Ok(known_labels_categories_map)
}

fn collect_args() -> clap::ArgMatches<'static> {
    App::new("CICA")
        .version("1.0")
        .author("Mathias LB <mathias.leborgne@gmail.com>")
        .about("Automatize expenses categories and sums for CIC bank accounts")
        .subcommand(SubCommand::with_name("guess")
                    .about("Guess categories from raw accounting file")
                    .arg(Arg::with_name("FILE")
                        .help("File name")
                        .required(true)
                        .index(1)
                        .takes_value(true)
                    )
                    .arg(Arg::with_name("MONTH")
                        .help("Month to check")
                        .required(true)
                        .index(2)
                        .takes_value(true)
                    )
                    .arg(Arg::with_name("YEAR")
                        .help("Year to check")
                        .required(true)
                        .index(3)
                        .takes_value(true)
                    ))
        .subcommand(SubCommand::with_name("sum")
                    .about("Sum categories from raw accounting file with categories")
                    .arg(Arg::with_name("FILE")
                        .help("File name")
                        .required(true)
                        .index(1)
                        .takes_value(true)))
        .get_matches()
}

fn replace_first_line(file_name: &str)-> Result<(), io::Error> {
    // https://stackoverflow.com/questions/27215396/how-to-replace-a-word-in-a-file-in-a-txt
    // https://stackoverflow.com/questions/27082848/rust-create-a-string-from-file-read-to-end

    let file_path = Path::new(&file_name);
    let mut file_content = Vec::new();
    {
        // can't just read as string as not-utf8 character appear (thanks CIC)
        let mut file = File::open(&file_path).expect("Unable to open file");
        file.read_to_end(&mut file_content).expect("Unable to read");
    }
    let position_newline = file_content.iter().position(| &x| x == 10).unwrap() + 1;
    // todo: replace unwrap?
    let rest_of_file = String::from_utf8((&file_content[position_newline..]).to_vec());
    let new_data = "Date,Datedevaleur,Montant,Libelle,Solde\n".to_string() + &rest_of_file.unwrap();
    // Recreate the file and dump the processed contents to it
    let mut dst = File::create(&file_path)?;
    dst.write(new_data.as_bytes())?;
    Ok(())
}

fn guess_accounting_entries_from_csv(file_name: &str, current_month: u32, year: i32) 
    -> Result<Vec<AccountingEntry>, csv::Error> {
    replace_first_line(&file_name)?;
    let known_labels_categories_map = get_known_labels_categories_map()?;
    let build_accounting_entry_from_raw_csv_record_with_cats = 
        |record: &HashMap<String, String>| 
        build_accounting_entry_from_raw_csv_record(record, &known_labels_categories_map);
    Ok(read_csv(&file_name, Some((current_month, year)), 
                &build_accounting_entry_from_raw_csv_record_with_cats)?)
}

fn guess_categories(file_name: &str, current_month: u32, year: i32) -> Result<(), csv::Error> {
    let accountings = guess_accounting_entries_from_csv(&file_name, current_month, year)?;
    print_accountings(&accountings, Some(current_month));
    let file_name_guessed = "guessed_".to_owned() + &file_name;
    match write_csv_guessed_categories(&accountings, &file_name_guessed) {
        Err(why) => panic!("Error when writing file: {:?}", why),
        Ok(_) => {},
    }; 
    println!("Modify {:?} and save it as something like {:?}", file_name_guessed.to_string(), 
             "account_guessed_categories_modified".to_string());
    Ok(())
}

fn main() -> Result<(), csv::Error> {
    let matches = collect_args();
    if let Some(sub_matches) = matches.subcommand_matches("guess") {
        let file_name = sub_matches.value_of("FILE").unwrap().to_string();
        let month = match sub_matches.value_of("MONTH").unwrap().parse::<u32>() {
            Ok(month) => month,
            Err(why) => panic!("month should be a number: {:?}", why),
        };
        let year = match sub_matches.value_of("YEAR").unwrap().parse::<i32>() {
            Ok(year) => year,
            Err(why) => panic!("year should be a number: {:?}", why),
        };
        println!("Balance: {:?}", read_balance_from_csv(&file_name, month, year));
        guess_categories(&file_name, month, year)?;
    }
    else if let Some(sub_matches) = matches.subcommand_matches("sum") {
        let file_name = sub_matches.value_of("FILE").unwrap().to_string();
        let accountings_modified = read_csv(&file_name, None,
                                            &build_accounting_entry_from_csv_record_with_categories)?;
        check_categories(&accountings_modified);
        print_accountings(&accountings_modified, None);
        let path_folder = Path::new(PATH_MODIFIED_ACCOUNTS);
        fs::copy(&file_name, path_folder.join(&file_name))?;
    }
    else {
        println!("Action should be guess or sum!"); // todo: better check
    }
    // todo: check usage of String vs &str
    Ok(())
}


// Automatic tests, mostly based on dummy accounting file "raw_account_2.csv"
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    const MONTH_TEST: u32 = 12;
    const YEAR_TEST: i32 = 2019;
    const FILE_NAME_TEST: &'static str = "raw_account_2.csv";

    fn get_test_accountings() -> Result<Vec<AccountingEntry>, csv::Error> {
        guess_accounting_entries_from_csv(&FILE_NAME_TEST.to_string(), MONTH_TEST, YEAR_TEST)
    }
    
    #[test]
    fn test_acquisition() {
        let accountings = get_test_accountings().unwrap();
        let accounting_entry = &accountings[0];
        assert_eq!(accounting_entry.amount, -1.60);       
    }
    
    #[test]
    fn test_length() {
        // NB: last entry is not in selected month/year, so it's excluded
        let accountings = get_test_accountings().unwrap();
        assert_eq!(accountings.len(), 5);       
    }
    
    #[test]
    fn test_auto_categories() {
        let accountings = get_test_accountings().unwrap();
        let accounting_entry = &accountings[0];
        assert_eq!(accounting_entry.category, "Voiture");        
    }

    #[test]
    fn test_auto_categories_modified() {
        let accountings = get_test_accountings().unwrap();
        let accounting_entry = &accountings[4];
        assert_eq!(accounting_entry.category, "Retraits");        
    }

    #[test]
    fn test_sum() {
        let accountings = get_test_accountings().unwrap();
        assert_eq!(get_sum_all_amounts(&accountings), -105.45);
    }

    #[test]
    fn test_balance_with_first_day_of_month() {
        let balance = read_balance_from_csv(&FILE_NAME_TEST.to_string(), MONTH_TEST, YEAR_TEST).unwrap();
        assert_eq!(balance, 7737.00);
    }
    
    #[test]
    fn test_balance_without_first_day_of_month() {
        let balance = read_balance_from_csv(&"raw_account_3.csv".to_string(), MONTH_TEST, YEAR_TEST).unwrap();
        assert_eq!(balance, 7738.23);
    }

}
