# Simtfl Rust

`simtfl-rust` is a rust implementaiton of the python implementation of [`simtfl`](https://github.com/electric-Coin-Company/simtfl).

## Goals of the project

- [x] Abstract: It simulates node behavior in response to messages based on the description of the protocol design. (It doesn’t need to make network connections, use persistent storage, or define message formats.)
- [x] Deterministic: A simulator run always produces identical results on any machine.
- [x] Network wide: It simulates the behavior of all nodes in the network, not just a single node.
- [x] Faster than real time: It simulates message transmission times, delays or message order interleaving directly without using the real-time clock.
- [x] Full network per message causal ordering: If node A sends a message `m_a` to node B, and node C sends a message `m_c` to node D, then it can simulate either `m_a` arriving before `m_c` or `m_c` arriving before `m_a`.

## Usage

The main demo is in the `bin/demo/src/main.rs` file. You can run it with the following command:

```bash
RUST_LOG=debug cargo run --bin demo
```

## Tests

You can run the all the tests with the following command:

```bash
cargo test
```

This will run all tests for goals except "deterministic". See [below](#test-goal-deterministic) for how to run the deterministic tests.

### Testing specific goals

#### Test goal: Abstract

```bash
cargo test --package demo --bin demo -- tests::test_abstract_behavior --exact --show-output
```

#### Test goal: Deterministic

```bash
chmod +x scripts/*.sh
./scripts/test_different_platforms.sh
```

#### Test goal: Network wide

```bash
cargo test --package demo --bin demo -- tests::test_network_wide_simulation --exact --show-output
```

#### Test goal: Faster than real time

```bash
cargo test --package demo --bin demo -- tests::test_faster_than_realtime --exact --show-output
```

#### Test goal: Full network per message causal ordering

```bash
cargo test --package demo --bin demo -- tests::test_message_ordering --exact --show-output
```
