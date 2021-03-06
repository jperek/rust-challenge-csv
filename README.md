# rust-challenge-csv

## Completeness
I attempted to handle all cases. I made the following assumptions:
* amount value for `available` or `held` can be negative, for example after charging back a deposit transaction with withdrawals in between.
* `locking` the account means the same as `freezing` the account which means that no withdrawals can be made, deposits and handling disputes works as usual.
* input transactions do not contain negative amounts
* only `Deposit` and `Withdrawal` transactions can be disputed

## Correctness
I tried to ensure correctnes of the application through unit tests. If I had more time, I'd write more elaborate unit tests and integration tests.
As the type for displaying `amount` I used i64, which assumes the maximum balance there can be is `922337203685477.5807` and the minimum one is equal to `-922337203685477.5808`. This makes sense for a currency with limited supply. Handling any case would require some kind of an arbitrary precision integer type.

## Safety and Robustness
Error handling is lackluster, instead of unwraps and expects the errors could be propagated to the highest abstraction level. I did not do that due to time constraints.

## Efficiency
I focused at making the application work according to the specification. These are performance improvements I could expect to see without doing any profiling:
* The app will probably be IO bound, due to reading from disk and not doing much of processing. For that reason the biggest performance improvement could be achieved through using a Direct IO API, which is available (`glommio`), but not that portable. I have used Directo IO in Windows with significant performance improvements on modern solid state drives.
* Use some kind of concurrency to allow the application to read from disk and process records at the same time, similarily for writing records to disk and serializing them to string representation. Just a thread and a channel could make significant impact.
* I expect many of the operations made in the app to allocate, which could certainly be avoided.
* Maybe if the system operated on a very high number of clients and a low number of transactions, a more fine-tuned structure could be selected. This would influence both CPU performance and memory usage
* I could also validate transactions as they come. At the moment I'm keeping all transactions in memory, I could check if they are valid keeping track of available resources for each client. That would be very beneficial in case of a large number invalid transactions, but in turn it would increase memory usage for cases where the number of clients is larger.

## Maintainability
I made a mistake of having too high ambitions. In some places I tried to avoid allocations, which led to messy code, for example in parsing the `Amount` from string. I could just take the string value, create a String object, remove the dot and parse as i64. I should've tried to write it the simplest way and improve on it later, especially so because I didn't avoid the allocations in many places as I run out of time.