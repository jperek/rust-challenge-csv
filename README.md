# rust-challenge-csv

## Completeness
I attempted to handle all cases. I made the following assumptions:
* amount value for `available` or `held` can be negative, for example after charging back a deposit transaction with withdrawals in between.
* `locking` the account means the same as `freezing` the account which means that no withdrawals can be made, deposits and handling disputes works as usual.

## Correctness
I tried to ensure correctnes of the application through unit tests. If I had more time, I'd write more elaborate unit tests and integration tests.
As the type for displaying `amount` I used i64, 

## Safety and Robustness