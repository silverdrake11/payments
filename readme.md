## Correctness
There are test cases at the end of the file. I tried to cover the major categories. They can be run with
```
cargo test
```
Some behavior is not defined, for example trying to resolve without a dispute. I ignored those cases. In addition it's not clear what happens when a locked account is trying to be modified. I assumed that when an account is locked, nothing can change it. Also sometimes _withdraw_ was used in the spec and other times _withdrawal_. I assumed it was _withdrawal_.

## Efficiency

The time complexity is O(transactions). Which is the number of rows in the CSV file.

The space complexity is O(transactions). Although the file is streaming, we need to keep track of the transactions in case they are referred to later, for example in the disputes. We could however only keep track of that subset of transactions, but then we'd have to read through the file again.

## Maintainability
The main data structure is the client which is stored in a HashMap keyed by the client ID. The client has three fields _available_,_held_, and _locked_. The total (_available_ + _held_) is not stored but is computed when needed.

The other data structure is a HashMap of transaction amounts (keyed by transaction ID). This is used to compute any values that need previous transaction amounts.