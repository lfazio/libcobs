# libcobs

Rust implementation of Consistent Overhead Byte Stuffing Protocol (COBS)

CobsSender encodes data on the fly before using a Sender trait to emit the encoded bytes.
CobsReceiver receives any byte length data into a `Vec<u8>` using a Receiver trait.

Using  `CobsSenderOperation` trait and `CobsReceiverOperation` trait empower the user 
to xmit/recv the encoded bytes on the interface he needs.

## Tests

Run the unit tets:

```sh
cargo test
```

## Coverage

### Text Output

```sh
cargo llvm-cov
```

### For Visual Studio Code

Using `Coverage Gutters` extension:

```sh
cargo llvm-cov --lcov --output-path lcov.info
```

> NOTE: you might need to click on `watch` in the bottom bar.