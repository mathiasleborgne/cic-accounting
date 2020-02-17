extern crate csv;
extern crate chrono;

use std::collections::HashMap;
use std::error::Error;
use std::env;
use std::fs;
use std::path::Path;
use std::fs::File;
use csv::Writer;
use std::io::{self, Read, Write};
use crate::chrono::Datelike; // todo: why?
use std::num::ParseIntError;
use regex::Regex;


const ALL_EXPENSE_CATEGORIES: [&str; 18] = [  // todo: put as global/const
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

fn get_category_from_label(label: &String, known_labels_categories_map: &HashMap<String, String>) -> String {
    match known_labels_categories_map.get(&get_label_without_number(label)) {
        Some(category) => return category.clone(),
        _ => return "Unknown".to_string(),
    }
}

fn build_accounting_entry_from_raw_csv_record(record: &HashMap<String, String>, known_labels_categories_map: &HashMap<String, String>) -> AccountingEntry {
    // todo: there might be a way to avoid cloning in here...
    AccountingEntry { 
        date_transaction: match chrono::NaiveDate::parse_from_str(&record["Date"].to_string(), "%m/%d/%Y") {
            Err(why) => panic!("{:?}", why),
            Ok(date_ok) => date_ok,
        }, 
        date_effect: record["Datedevaleur"].clone(),
        amount: match record["Montant"].parse::<f32>() {
            Err(why) => panic!("{:?}", why),
            Ok(amount_float) => amount_float,
        }, 
        label: record["Libelle"].clone(),
        category: get_category_from_label(&record["Libelle"], &known_labels_categories_map),
    }
}

fn build_accounting_entry_from_csv_record_with_categories(record: &HashMap<String, String>) -> AccountingEntry {
    // todo: there might be a way to avoid cloning in here...
    AccountingEntry { 
        date_transaction: match chrono::NaiveDate::parse_from_str(&record["Date"].to_string(), "%m/%d/%Y") {
            Err(why) => panic!("{:?}", why),
            Ok(date_ok) => date_ok,
        }, 
        date_effect: record["Datedevaleur"].clone(),
        amount: match record["Montant"].parse::<f32>() {
            Err(why) => panic!("{:?}", why),
            Ok(amount_float) => amount_float,
        }, 
        label: record["Libelle"].clone(),
        category: record["Category"].clone(),
    }
}

fn get_sum_all_amounts(accountings: &Vec<AccountingEntry>, current_month: u32) -> f32 {
    let mut sum_all_amounts = 0.; 
    for accounting_entry in accountings {
        if accounting_entry.date_transaction.month() == current_month {
            sum_all_amounts += accounting_entry.amount;
        }
    }
    return sum_all_amounts
}

fn get_sum_category(accountings: &Vec<AccountingEntry>, category: String, current_month: u32) -> f32 {
    // todo: not so clean
    let mut sum_category = 0.;
    for accounting_entry in accountings {
        if accounting_entry.category == category && accounting_entry.date_transaction.month() == current_month {
            sum_category += accounting_entry.amount;
        }
    }
    return sum_category
}

fn write_csv_guessed_categories(accountings: &Vec<AccountingEntry>, file_name_guessed: &String) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(file_name_guessed)?;
    wtr.write_record(&["Date", "Datedevaleur", "Montant", "Libelle", "Category"]);
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
    
fn read_csv(csv_path: &String, month: Option<u32>, year: Option<i32>, entry_builder_function: &dyn Fn(&HashMap<String, String>)-> AccountingEntry) 
    -> Result<Vec<AccountingEntry>, csv::Error> {
    // pass an entry_builder_function to read csv with or without categories
    // https://www.reddit.com/r/rust/comments/bwplfl/read_csv_columns/
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut accountings: Vec<AccountingEntry> = Vec::new();
    for result in rdr.deserialize() {
        let record: HashMap<String, String> = result?;
        let accounting_entry = entry_builder_function(&record);
        // todo: make Pair month, year optional
        match month {
            None => accountings.push(accounting_entry),
            Some(month) => {
                match year  {
                    None => accountings.push(accounting_entry),
                    Some(year) => {
                        if accounting_entry.date_transaction.month() == month && accounting_entry.date_transaction.year() == year {
                            accountings.push(accounting_entry);
                        }
                    }
                }
            },
        }
    }
    return Ok(accountings)
}

fn print_accountings(accountings: &Vec<AccountingEntry>, current_month: u32) {
    // println!("{:#?}", accountings);
    println!("----------------");
    for expense_category in ALL_EXPENSE_CATEGORIES.iter() {        
        println!("{:?} expenses: {:?}", expense_category, get_sum_category(&accountings, expense_category.to_string(), current_month));
    }
    println!("----------------");
    println!("Total expenses: {:?}", get_sum_all_amounts(&accountings, current_month));
    println!("");
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
                let accountings_guessed_file = read_csv(&path_str.to_string(), None, None, &build_accounting_entry_from_csv_record_with_categories)?;
                for accounting_guessed in accountings_guessed_file {
                    known_labels_categories_map.insert(get_label_without_number(&accounting_guessed.label.clone()) , accounting_guessed.category.clone());
                }
            },
        }
    }
    // println!("Labels map: {:?}", known_labels_categories_map);
    // example: 
    // known_labels_categories_map.insert(get_label_without_number(&"PAIEMENT CB  CHAVILLE MONOPRIX CARTE ".to_string()), "Courses".to_string());
    Ok(known_labels_categories_map)
}

fn collect_args() -> Result<(u32, i32, String, String), ParseIntError> {
    // action/file_name/month/year
    // todo: check length of args?
    // todo: use crate?
    let args: Vec<String> = env::args().collect();
    let action = args[1].to_string();
    let file_name = args[2].to_string();
    let month = args[3].parse::<u32>()?; 
    let year = args[4].parse::<i32>()?; 

    Ok((month, year, action, file_name))
}

fn replace_first_line(file_name: &String)-> Result<(), io::Error> {
    // https://stackoverflow.com/questions/27215396/how-to-replace-a-word-in-a-file-in-a-txt
    // todo: fix data.replace
    let file_path = Path::new(&file_name);

    // Open and read the file entirely
    let mut data = String::new();
    {
        let mut src = File::open(&file_path)?;
        src.read_to_string(&mut data)?;
    }
    // Run the replace operation in memory
    let new_data = data.replace("Date,Date de valeur,Montant,Libell�,Solde", "Date,Datedevaleur,Montant,Libelle,Solde");

    // Recreate the file and dump the processed contents to it
    let mut dst = File::create(&file_path)?;
    dst.write(new_data.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), csv::Error> {
    let (current_month, year, action, file_name) = match collect_args() {
        Err(why) => panic!("Error when collecting arguments, try somethin like \"cargo run 12 2019 guess dummy.csv\": {:?}", why),
        Ok(tuple_result) => tuple_result,
    };
    match action.as_ref() {
        "guess" => {
            replace_first_line(&file_name.to_string());
            let known_labels_categories_map = get_known_labels_categories_map()?;
            let build_accounting_entry_from_raw_csv_record_with_cats = 
                |record: &HashMap<String, String>| 
                build_accounting_entry_from_raw_csv_record(record, &known_labels_categories_map);
            let accountings = read_csv(&file_name, Some(current_month), Some(year), &build_accounting_entry_from_raw_csv_record_with_cats)?;
            print_accountings(&accountings, current_month);
            let file_name_guessed = "guessed_".to_owned() + &file_name;
            write_csv_guessed_categories(&accountings, &file_name_guessed); 
            println!("Modify {:?} and save it as {:?}", file_name_guessed.to_string(), "account_guessed_categories_modified".to_string());
        },
        "sum" => {
            let accountings_modified = read_csv(&file_name, Some(current_month), Some(year), &build_accounting_entry_from_csv_record_with_categories)?;
            print_accountings(&accountings_modified, current_month);
            let path_folder = Path::new(PATH_MODIFIED_ACCOUNTS);
            fs::copy(&file_name, path_folder.join(&file_name))?;
        },
        _ => println!("Action should be guess or sum!"), // todo: better check
    }
        // todo: replace 1st line
        // todo: remove month check from sums funcitons
        // todo: remove unused args for sum action
        // todo: retraitsSO/P should be Retraits
    Ok(())
}

