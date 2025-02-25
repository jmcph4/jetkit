# Transaction Inclusion Tool #

`tit` is the **T**ransaction **I**nclusion **T**ool. It forwards a signed Ethereum transaction to one or more Execution Layer nodes.

## Usage ##

```
Gets transactions onto the Ethereum blockchain

Usage: tit [OPTIONS] <TX>

Arguments:
  <TX>  Hexadecimal string representing valid transaction RLP bytes

Options:
  -r, --rpcs <RPCS>  List of endpoints to submit the transaction to
  -s, --strict       Attempt to decode the RLP bytes into an EIP-2718 envelope and halt if this fails
  -q, --quiet        Do not print to standard output
  -p, --private      Use `eth_sendPrivateRawTransaction`
  -h, --help         Print help
  -V, --version      Print version
```

### Examples ###

#### Sending a transaction to a single node ####

```
$ tit -r https://rpc.beaverbuild.org 0x02f8b20181948449bdee618501dcd6500083016b93942dabcea55a12d73191aece59f508b191fb68adac80b844095ea7b300000000000000000000000054e44dbb92dba848ace27f44c0cb4268981ef1cc00000000000000000000000000000000000000000000000052616e065f6915ebc080a0c497b6e53d7cb78e68c37f6186c8bb9e1b8a55c3e22462163495979b25c2caafa052769811779f438b73159c4cc6a05a889da8c1a16e432c2e37e3415c9a0b9887
https://rpc.beaverbuild.org/ said: {"id":1,"jsonrpc":"2.0","result":"0xed996b9391ba983bad386a33dfc2eb91059c4a322b685246f9937cbbf9d5ad49"}
```

#### Sending a transaction to a multiple nodes ####

```
$ tit -r https://rpc.beaverbuild.org -r https://rpc.titanbuilder.xyz -r https://rpc.penguinbuild.org 0x02f8b20181948449bdee618501dcd6500083016b93942dabcea55a12d73191aece59f508b191fb68adac80b844095ea7b300000000000000000000000054e44dbb92dba848ace27f44c0cb4268981ef1cc00000000000000000000000000000000000000000000000052616e065f6915ebc080a0c497b6e53d7cb78e68c37f6186c8bb9e1b8a55c3e22462163495979b25c2caafa052769811779f438b73159c4cc6a05a889da8c1a16e432c2e37e3415c9a0b9887
https://rpc.titanbuilder.xyz/ said: {"jsonrpc":"2.0","id":1,"result":null}
https://rpc.beaverbuild.org/ said: {"id":1,"jsonrpc":"2.0","result":"0xed996b9391ba983bad386a33dfc2eb91059c4a322b685246f9937cbbf9d5ad49"}
https://rpc.penguinbuild.org/ said: error code: 522
```

#### Sending a private transaction to a single node ####

```
$ tit -p -r https://rpc.beaverbuild.org 0x02f8b20181948449bdee618501dcd6500083016b93942dabcea55a12d73191aece59f508b191fb68adac80b844095ea7b300000000000000000000000054e44dbb92dba848ace27f44c0cb4268981ef1cc00000000000000000000000000000000000000000000000052616e065f6915ebc080a0c497b6e53d7cb78e68c37f6186c8bb9e1b8a55c3e22462163495979b25c2caafa052769811779f438b73159c4cc6a05a889da8c1a16e432c2e37e3415c9a0b9887
https://rpc.beaverbuild.org/ said: {"id":1,"jsonrpc":"2.0","result":"0xed996b9391ba983bad386a33dfc2eb91059c4a322b685246f9937cbbf9d5ad49"}
```

