# cic-accounting

Rust code to manage accounting records in CSV format.

    cargo run <action (guess/sum)> <file_name> <month> <year> 
    cargo run guess accounts_12_2019.csv 12 2019 
    # modify guessed_accounts_12_2019.csv, save as modified_accounts_12_2019.csv
    cargo run sum modified_accounts_12_2019.csv 12 2019 
