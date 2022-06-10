use std::{collections::HashMap, io};

use crate::account::Account;

pub type TransactionID = u32;
pub type ClientID = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub ty: TransactionType,
    pub client_id: ClientID,
    pub id: TransactionID,
    pub amount: f32,
}

impl Transaction {
    fn parse(input: &str) -> Result<Option<Self>, &'static str> {
        let mut columns = input.split(',');

        let transaction_ty = if let Some(type_str) = columns.next() {
            let trimmed_type_str = type_str.trim();
            if trimmed_type_str.is_empty() {
                return Ok(None);
            }
            if let Ok(ty) = TransactionType::try_from(type_str.trim()) {
                ty
            } else {
                return Err("invalid transaction type");
            }
        } else {
            return Err("no transaction type");
        };

        let transaction = Transaction {
            ty: transaction_ty,
            client_id: columns
                .next()
                .ok_or("no client ID")?
                .trim()
                .parse::<ClientID>()
                .map_err(|_| "invalid client ID")?,
            id: columns
                .next()
                .ok_or("no transaction ID")?
                .trim()
                .parse::<TransactionID>()
                .map_err(|_| "invalid transaction ID")?,
            amount: columns
                .next()
                .unwrap_or("0")
                .trim()
                .parse::<f32>()
                .unwrap_or(0.0),
        };

        Ok(Some(transaction))
    }

    pub fn process(
        &self,
        account: &mut Account,
        past_transactions: &HashMap<TransactionID, Transaction>,
    ) {
        use TransactionType::*;

        match self.ty {
            Deposit => {
                account.available += self.amount;
                account.total += self.amount;
            }
            Withdrawal => {
                let result = account.available - self.amount;

                if result > 0.0 {
                    account.available = result;
                    account.total = result;
                }
            }
            Dispute => {
                debug_assert_eq!(self.amount, 0.0);
                if let Some(transaction) = past_transactions.get(&self.id) {
                    let disputed_amount = transaction.amount;
                    account.available -= disputed_amount;
                    account.held += disputed_amount;
                } else {
                    // We will assume this is an error on the partner's side
                }
            }
            Resolve => {
                debug_assert_eq!(self.amount, 0.0);
                if let Some(transaction) = past_transactions.get(&self.id) {
                    if transaction.ty == TransactionType::Dispute {
                        let non_disputed_amount = transaction.amount;
                        account.held -= non_disputed_amount;
                        account.available += non_disputed_amount;
                    }
                }
                // Otherwise we will assume this is an error on the partner's side
            }
            Chargeback => {
                debug_assert_eq!(self.amount, 0.0);
                if let Some(transaction) = past_transactions.get(&self.id) {
                    if transaction.ty == TransactionType::Dispute {
                        let disputed_amount = transaction.amount;
                        account.held -= disputed_amount;
                        account.total -= disputed_amount;
                        account.locked = true;
                    }
                }
                // Otherwise we will assume this is an error on the partner's side
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryFrom<&str> for TransactionType {
    type Error = ();

    fn try_from(other: &str) -> Result<Self, Self::Error> {
        use TransactionType::*;

        Ok(match other {
            "deposit" => Deposit,
            "withdrawal" => Withdrawal,
            "dispute" => Dispute,
            "resolve" => Resolve,
            "chargeback" => Chargeback,
            _ => return Err(()),
        })
    }
}

pub fn parse_transactions(reader: impl io::BufRead) -> Result<Vec<Transaction>, &'static str> {
    // As opposed to loading all data into memory
    // this will reuse a single buffer to process all data
    let mut rows = reader.lines();

    rows.next(); // Skip row of column types

    let mut transactions = Vec::<Transaction>::new();
    for row in rows {
        if let Ok(row) = row {
            if let Some(transaction) = Transaction::parse(&row)? {
                transactions.push(transaction);
            }
        } else {
            return Err("failed reading row");
        }
    }

    Ok(transactions)
}
