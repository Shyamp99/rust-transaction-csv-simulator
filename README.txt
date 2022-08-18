To Run program: use 
    cargo run -- path_to_input_csv > path_to_output_csv

Assumptions:
- transactions.csv is prechecked by partner and there isn't malformed data: i.e a string in the place where a float should be and Dispute, Chargeback and Resolve rows are formatted as "transaction,client,tx, "
- Someone will not dispute a withdrawal because logically when you withdraw, you are taking your own money out of the account and putting it elsewhere
- Chargeback and Resolves cannot occur if a Disupte hasn't been placed first and after a Resolve, or Chargeback a transaction's status gets reset to no longer be disputed
- Chargeback and Resolves are separate actions (I'm not super familiar with the general dispute process)


Limitations:
- Due to my limited rust knowledge, I was not able to get Serde to handle scenarios where datapoints may have random whitespace before, after or in between digits.
    - i.e a row like "withdrawal,  1,15 ,0.01"
    - Ignore rows serde cannot deserialize into an object
    - This was due to my first time using Serde and my limited Rust knowledge of prechecking or correcting CSV rows
        - That being said, I am excited at the prospect of being able to learn Rust at an enterprise level
- Truncating via the built in f64::trunc() and formatting with "{:.4}" always seems round the number at the 4th decimal place but you do not want to charge the client more than what they requested. The alternative approach would be to turn it to a string, take the substring and then convert to a float again which would be extremely inefficent, so I am opting to allow the rounding at the 4th decimal place since it goes to a ten - thousandth of a unit.

Possible Improvements due to limiting myself from spending too much time:
- Designing this to be much more object oriented as opposed to just using structs and static functions

Testing:
- For testing I wrote out a couple of csv files and ran a basic command along the lines of:
        for input_file in ./csv_dir/*; cargo run -- input_file > input_file_output.csv; done

General idea:
- Using Serde to parse the input csv and ignore any malformed rows
- Serde reads the data into structs which are stored into a vector and returned
- Then we iterate over the vector and based on each transaction we either create a new client struct or modify an existing one: the client structs are stored in a hashmap where key = client_id and value = client struct
- For disputes, chargebacks, and resolves, we reference the disputes hasmap in each client struct where key = disputed tx and value = transaction amount
- Finally, go through hashmap after looking at the final transaction and write the csv to stdout
