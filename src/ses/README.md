# Simple Ethereum Signer #

`ses` is the **S**imple **E**thereum **S**igner. It can produce new Ethereum private keys and display information about existing ones (like the associated Ethereum address or the associated public key).

## Usage ##

```
Generates a SECP-256k1 keypair or displays the Ethereum address associated with an existing one

Usage: ses [OPTIONS] [OUT]

Arguments:
  [OUT]  Path to save generated private key to

Options:
  -q, --quiet               Do not write to standard output
  -d, --display-public-key  Display the public key instead of the Ethereum address
  -i, --in <IN>             Path to private key
  -h, --help                Print help
  -V, --version             Print version
```

