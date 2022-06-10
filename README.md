# transactions

A toy payments engine that reads transaction input from a CSV, handles
deposits, withdrawals, disputes, resolves, and chargebacks, and finally
serializes the resulting data back to CSV.

## Building and running

```
$ cargo run -- transactions.csv > result.csv
```