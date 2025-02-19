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

### Examples ###

#### Calculating the Ethereum address associated with a private key ####

```
$ python3.11 -c "import sys; sys.stdout.buffer.write(bytes.fromhex(sys.argv[1]))"  0000000000000000000000000000000000000000000000000000000000000001  > well_known_key.secret
$ ses -i well_known_key.secret
0x7e5f4552091a69125d5dfcb7b8c2659029395bdf
```

#### Calculating the public key associated with a private key ####

```
$ python3.11 -c "import sys; sys.stdout.buffer.write(bytes.fromhex(sys.argv[1]))"  0000000000000000000000000000000000000000000000000000000000000001  > well_known_key.secret
$ ses -di well_known_key.secret
0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8
```

#### Generating a new private key ####

**Please do not use this private key!**

```
$ ses new_private_key.secret
0xa326660309ded56b4ff8451cfc00fa8d5598d332
$ xxd new_private_key.secret
00000000: 2f8d 26d4 4daa cd46 0965 7816 4bb3 1c5c  /.&.M..F.ex.K..\
00000010: 94be 11d7 4716 a84a a297 cb0b 401e 0c05  ....G..J....@...
```

