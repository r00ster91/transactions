use std::collections::HashMap;

use crate::transaction::ClientID;

#[derive(Debug, Default)]
pub struct Account {
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl Account {
    fn serialize(&self, client_id: ClientID) -> String {
        format!(
            "{},{},{},{},{}\n",
            client_id, self.available, self.held, self.total, self.locked
        )
    }
}

pub fn serialize_accounts(accounts: &HashMap<ClientID, Account>) -> String {
    let mut string = String::new();
    string.push_str("client,available,held,total,locked\n");
    for (client_id, account) in accounts.iter() {
        string.push_str(&account.serialize(*client_id));
    }
    string
}
