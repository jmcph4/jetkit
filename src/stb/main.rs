use std::{
    fmt, fs,
    io::{self, Write},
    path::PathBuf,
};

use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_primitives::{Address, ChainId, U256};
use alloy_rpc_types::TransactionRequest;
use alloy_signer_local::LocalSigner;
use clap::Parser;
use eyre::eyre;
use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct TransactionSpecification {
    pub r#type: Option<u8>,
    pub to: Option<Address>,
    pub input: Option<Vec<u8>>,
    pub value: Option<U256>,
    pub chain_id: Option<ChainId>,
    pub nonce: Option<u64>,
    pub gas_limit: Option<u64>,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
}

impl From<TransactionSpecification> for TransactionRequest {
    fn from(val: TransactionSpecification) -> Self {
        TransactionRequest::default()
            .transaction_type(val.r#type.unwrap_or_default())
            .to(val.to.unwrap_or_default())
            .input(val.input.unwrap_or_default().into())
            .value(val.value.unwrap_or_default())
            .nonce(val.nonce.unwrap_or_default())
            .gas_limit(val.gas_limit.unwrap_or_default())
            .max_fee_per_gas(val.max_fee_per_gas.unwrap_or_default())
            .max_priority_fee_per_gas(
                val.max_priority_fee_per_gas.unwrap_or_default(),
            )
    }
}

impl fmt::Display for TransactionSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Type: {}",
            match self.r#type.unwrap_or_default() {
                0 => "0x00 (Legacy)",
                1 => "0x01 (Access List)",
                2 => "0x02 (Fee Market)",
                3 => "0x03 (Blob)",
                _ => "(illegal)",
            }
        )?;
        writeln!(f, "To: {}", self.to.unwrap_or_default())?;
        writeln!(f, "Value: {}", self.value.unwrap_or_default())?;
        writeln!(
            f,
            "Input: {}",
            hex::encode(self.input.as_ref().unwrap_or(&vec![]))
        )?;
        Ok(())
    }
}

pub fn input(prompt: &str) -> eyre::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn interactive() -> eyre::Result<TransactionSpecification> {
    let mut spec = TransactionSpecification::default();
    let tx_type: u8 = input("EIP-2718 transaction envelope type (0 = Legacy, 1 = Access List, 2 = Fee Market, 3 = Blob): ")?.parse()?;
    spec.r#type = Some(tx_type);
    let to: Address = input("To: ")?.parse()?;
    spec.to = Some(to);
    let value: U256 = input("Value: ")?.parse()?;
    spec.value = Some(value);
    let data: Vec<u8> = hex::decode(&input("Input: ")?[2..])?;
    spec.input = Some(data);
    let nonce: u64 = input("Nonce: ")?.parse()?;
    spec.nonce = Some(nonce);
    let gas_limit: u64 = input("Gas Limit: ")?.parse()?;
    spec.gas_limit = Some(gas_limit);

    if tx_type >= 2 {
        let max_fee_per_gas: u128 = input("Maximum Fee per Gas: ")?.parse()?;
        spec.max_fee_per_gas = Some(max_fee_per_gas);
        let max_priority_fee_per_gas: u128 =
            input("Maximum Priority Fee per Gas: ")?.parse()?;
        spec.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
    }

    Ok(spec)
}

/// Produces Ethereum transactions from structured specifications
#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
struct Opts {
    /// Use an interactive prompt instead of a specification file for defining
    /// the transaction
    #[clap(short, long, action)]
    pub interactive: bool,
    /// Path to a specification file defining the transaction
    #[arg(required_unless_present = "interactive")]
    pub spec: Option<PathBuf>,
    #[clap(short, long)]
    pub quiet: bool,
    /// When reading or writing private keys, interpret them as 0x-prefixed
    /// UTF-8-encoded hexadecimal strings (unless `--no-hex-prefix` is
    /// specified)
    #[clap(long, short = 'a')]
    pub human_readable: bool,
    /// When reading or writing private keys in human-readable, hexadecimal
    /// form, do not use 0x-prefixes
    #[clap(long, short = 'b')]
    pub no_hex_prefix: bool,
    /// In interactive mode, write the specification file format to standard
    /// output
    #[clap(long, short)]
    pub dump_spec: bool,
    /// Path to private key
    #[clap(long, short)]
    #[arg(required_unless_present = "dump_spec")]
    pub private_key: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let opts = Opts::parse();

    let spec: TransactionSpecification = if opts.interactive {
        interactive()?
    } else {
        serde_json::from_str(&fs::read_to_string(opts.spec.unwrap())?)?
    };

    if opts.interactive && opts.dump_spec && !opts.quiet {
        println!("{}", serde_json::to_string(&spec)?);
    }

    /* there's no real point in printing the spec itself if we've just dumped
     * its JSON */
    if !opts.quiet && !opts.dump_spec {
        print!("{spec}");
    }

    if let Some(private_key) = opts.private_key {
        let private_key_bytes = if opts.human_readable {
            if opts.no_hex_prefix {
                hex::decode(fs::read(private_key)?)?
            } else {
                hex::decode(&fs::read(private_key)?[2..])?
            }
        } else {
            fs::read(private_key)?
        };
        let private_key_slice: &[u8; 32] =
            match private_key_bytes.as_slice().try_into() {
                Ok(t) => t,
                Err(e) => {
                    return Err(eyre!(
                        "Incorrect data length for private key: {e:?}"
                    ))
                }
            };

        let wallet: EthereumWallet =
            EthereumWallet::new(LocalSigner::from_signing_key(
                SigningKey::from_slice(private_key_slice)?,
            ));

        println!(
            "0x{}",
            hex::encode(alloy_rlp::encode(
                &<TransactionSpecification as Into<TransactionRequest>>::into(
                    spec
                )
                .build(&wallet)
                .await?
            ))
        );
    }

    Ok(())
}
