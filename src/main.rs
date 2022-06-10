use std::{collections::HashMap, env, fs, io, process::ExitCode};

mod account;
mod transaction;

use account::{serialize_accounts, Account};
use transaction::{parse_transactions, ClientID, Transaction, TransactionID};

fn main() -> ExitCode {
    let mut args = env::args();

    args.next(); // Skip program name

    if let Some(arg) = args.next() {
        if let Ok(file) = fs::File::open(&arg) {
            let reader = io::BufReader::new(file);
            match parse_transactions(reader) {
                Ok(transactions) => {
                    let accounts = handle_transactions(&transactions);
                    let output = serialize_accounts(&accounts);
                    print!("{output}");
                    return ExitCode::from(0);
                }
                Err(err) => {
                    eprintln!("transactions could not be parsed: {}", err);
                }
            }
        } else {
            eprintln!("could not read transactions CSV file!");
        }
    } else {
        eprintln!("no CSV file of transactions provided!");
    }
    ExitCode::from(1)
}

fn handle_transactions(transactions: &[Transaction]) -> HashMap<ClientID, Account> {
    let mut accounts = HashMap::<ClientID, Account>::new();
    let mut processed_transactions =
        HashMap::<TransactionID, Transaction>::with_capacity(transactions.len());

    for transaction in transactions {
        let account = accounts
            .entry(transaction.client_id)
            .or_insert_with(Account::default);
        transaction.process(account, &processed_transactions);
        processed_transactions.insert(transaction.id, transaction.clone());
    }

    accounts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_accounts_integrity<'a>(accounts: impl Iterator<Item = &'a Account>) {
        for account in accounts {
            assert_eq!(account.available, account.total - account.held);
            assert_eq!(account.held, account.total - account.available);
            assert_eq!(account.total, account.available + account.held);
        }
    }

    #[test]
    fn it_deposits_and_withdraws() {
        let transactions_string = "type,       client, tx,  amount\n\
                                         deposit,    5,      100, 10.0\n\
                                         deposit,    10,     2,   39.99\n\
                                         deposit,    20,     3,   50.0\n\
                                         withdrawal, 5,      4,   2.5\n\
                                         withdrawal, 10,     5,   1.0\n\
                                         withdrawal, 20,     6,   1.0\n\
                                         ";
        let transactions = parse_transactions(io::Cursor::new(transactions_string)).unwrap();

        use transaction::TransactionType::*;
        assert_eq!(
            transactions,
            [
                Transaction {
                    ty: Deposit,
                    client_id: 5,
                    id: 100,
                    amount: 10.0,
                },
                Transaction {
                    ty: Deposit,
                    client_id: 10,
                    id: 2,
                    amount: 39.99,
                },
                Transaction {
                    ty: Deposit,
                    client_id: 20,
                    id: 3,
                    amount: 50.0,
                },
                Transaction {
                    ty: Withdrawal,
                    client_id: 5,
                    id: 4,
                    amount: 2.5,
                },
                Transaction {
                    ty: Withdrawal,
                    client_id: 10,
                    id: 5,
                    amount: 1.0,
                },
                Transaction {
                    ty: Withdrawal,
                    client_id: 20,
                    id: 6,
                    amount: 1.0,
                },
            ]
        );

        let accounts = handle_transactions(&transactions);

        assert!(accounts.len() == 3);
        test_accounts_integrity(accounts.values());

        // We have no guaranteed account order
        let output = serialize_accounts(&accounts);
        assert!(output.starts_with("client,available,held,total,locked"));
        assert!(output.contains("5,7.5,0,7.5,false"));
        assert!(output.contains("20,49,0,49,false"));
        assert!(output.contains("10,38.99,0,38.99,false"));
    }

    #[test]
    fn it_handles_disputes() {
        let transactions_string = "type,    client, tx,  amount\n\
                                         deposit, 5,      100, 10.0\n\
                                         dispute, 5,      101\n\
                                         resolve, 5,      101\n\
                                         ";
        let transactions = parse_transactions(io::Cursor::new(transactions_string)).unwrap();

        use transaction::TransactionType::*;
        assert_eq!(
            transactions,
            [
                Transaction {
                    ty: Deposit,
                    client_id: 5,
                    id: 100,
                    amount: 10.0,
                },
                Transaction {
                    ty: Dispute,
                    client_id: 5,
                    id: 101,
                    amount: 0.0,
                },
                Transaction {
                    ty: Resolve,
                    client_id: 5,
                    id: 101,
                    amount: 0.0,
                },
            ]
        );

        let accounts = handle_transactions(&transactions);

        assert!(accounts.len() == 1);
        test_accounts_integrity(accounts.values());

        let output = serialize_accounts(&accounts);
        assert!(output.starts_with("client,available,held,total,locked"));
        assert!(output.contains("5,10,0,10,false\n"));
    }

    #[test]
    fn it_handles_chargebacks() {
        let transactions_string = "type,      client, tx,  amount\n\
                                         deposit,    10,      2, 99.9999\n\
                                         dispute,    10,      3,\n\
                                         chargeback, 10,      3,\n\
                                         ";
        let transactions = parse_transactions(io::Cursor::new(transactions_string)).unwrap();

        use transaction::TransactionType::*;
        assert_eq!(
            transactions,
            [
                Transaction {
                    ty: Deposit,
                    client_id: 10,
                    id: 2,
                    amount: 99.9999,
                },
                Transaction {
                    ty: Dispute,
                    client_id: 10,
                    id: 3,
                    amount: 0.0,
                },
                Transaction {
                    ty: Chargeback,
                    client_id: 10,
                    id: 3,
                    amount: 0.0,
                },
            ]
        );

        let accounts = handle_transactions(&transactions);

        assert!(accounts.len() == 1);
        test_accounts_integrity(accounts.values());
        assert!(accounts.values().next().unwrap().locked);

        let output = serialize_accounts(&accounts);
        assert!(output.starts_with("client,available,held,total,locked"));
        assert!(output.contains("10,99.9999,0,99.9999,true\n"));
    }
}
