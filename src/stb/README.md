# Structured Transaction Builder #

`stb` is the **S**tructured **T**ransaction **B**uilder. It serialises Ethereum transactions based on human-readable specifications (either from a configuration file or standard input).

## Usage ##

```
Produces Ethereum transactions from structured specifications

Usage: stb [OPTIONS] [SPEC]

Arguments:
  [SPEC]  Path to a specification file defining the transaction

Options:
  -i, --interactive                Use an interactive prompt instead of a specification file for defining the transaction
  -q, --quiet                      
  -a, --human-readable             When reading or writing private keys, interpret them as 0x-prefixed UTF-8-encoded hexadecimal strings (unless `--no-hex-prefix` is specified)
  -b, --no-hex-prefix              When reading or writing private keys in human-readable, hexadecimal form, do not use 0x-prefixes
  -d, --dump-spec                  In interactive mode, write the specification file format to standard output
  -p, --private-key <PRIVATE_KEY>  Path to private key
  -h, --help                       Print help
  -V, --version                    Print version
```

### Examples ###

#### Create a transaction from a configuration file ####

```
$ ses private_key.secret
$ stb -q -p private_key.secret spec.json
0xb86302f860010180806494ee44b533c827a5a70141e00284e1916d4e0d205b8000c001a0bc218b6ff948b367005f06c388c60487079feb4f5a3ea4eaa0ec3063997b622fa037e51cd39cd2f00e77e552375dad4bb9abe0708dc18359e0413d1fb6da15ca63
```

#### Create a transaction from an interactive prompt ####

```
$ ses private_key.secret
$ stb -p private_key.secret -iq
EIP-2718 transaction envelope type (0 = Legacy, 1 = Access List, 2 = Fee Market, 3 = Blob): 0
To: 0xeE44B533c827A5A70141E00284e1916D4e0d205B
Value: 100
Input: 0x00
Nonce: 1
Gas Limit: 30000000
0xb86702f864010180808401c9c38094ee44b533c827a5a70141e00284e1916d4e0d205b6400c001a0cec4f9251ee1b3dbdbda8f6660852c93d4a480fd5cb5ea950c459546e225323da061eb893fe59515b5e9328f7e31d90f6e9f51c8044921c5ed1b2b9087b672d394
```

#### Create a configuration file from an interactive prompt ####

```
$ stb -id
EIP-2718 transaction envelope type (0 = Legacy, 1 = Access List, 2 = Fee Market, 3 = Blob): 0
To: 0xeE44B533c827A5A70141E00284e1916D4e0d205B
Value: 100
Input: 0x00
Nonce: 1
Gas Limit: 30000000
{"type":0,"to":"0xee44b533c827a5a70141e00284e1916d4e0d205b","input":[0],"value":"0x64","nonce":1,"gas_limit":30000000}
```

