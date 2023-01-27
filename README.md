# Matching Engine

Basic order matching engine system written in Rust. 

## Disclaimer

The features provided by this draft implementation are not meant to be functionally complete and are not suitable for deployment in production.

**Use this software at your own risk.**


## Features

Supported features:

- [x] market orders
- [x] limit orders
- [x] stop orders
- [x] stop limit orders
- [ ] trailing stop orders
- [ ] trailing stop limit orders
- [ ] limit maker

## Usage

An example code using [redb](https://github.com/cberner/redb) could be found in [examples/redb_example.rs](./examples/redb_example.rs).
This example supports ACID transactions for each order.


## Benchmarks

TBD
