# DAO Certificate With Spore

Run tests:
```bash
$ CARGO_ARGS="--features unitest --no-default-features" make build
$ make test
```

Build for `testnet`:
```bash
$ CARGO_ARGS="--features testnet --no-default-features" make build
```

Build for `mainnet`:
```bash
$ make build
```
