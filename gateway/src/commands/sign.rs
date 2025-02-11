// Copyright 2020 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::commands::*;
use crate::config::{persistence::pathfinder::GatewayPathfinder, Config};
use clap::{App, Arg, ArgMatches};
use colored::Colorize;
#[cfg(feature = "coconut")]
use config::defaults::BECH32_PREFIX;
use config::NymConfig;
use crypto::asymmetric::identity;
use log::error;
use std::process;
#[cfg(feature = "coconut")]
use subtle_encoding::bech32;
#[cfg(not(feature = "coconut"))]
use validator_client::nymd::AccountId;

const SIGN_TEXT_ARG_NAME: &str = "text";
const SIGN_ADDRESS_ARG_NAME: &str = "address";

pub fn command_args<'a, 'b>() -> App<'a, 'b> {
    let cmd = App::new("sign")
        .about("Sign text to prove ownership of this mixnode")
        .arg(
            Arg::with_name(ID_ARG_NAME)
                .long(ID_ARG_NAME)
                .help("The id of the mixnode you want to sign with")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(SIGN_TEXT_ARG_NAME)
                .long(SIGN_TEXT_ARG_NAME)
                .help("Signs an arbitrary piece of text with your identity key")
                .takes_value(true)
                .conflicts_with(SIGN_ADDRESS_ARG_NAME),
        );

    let mut address_sign_cmd = Arg::with_name(SIGN_ADDRESS_ARG_NAME)
        .long(SIGN_ADDRESS_ARG_NAME)
        .help("Signs your blockchain address with your identity key")
        .conflicts_with(SIGN_TEXT_ARG_NAME);

    if cfg!(feature = "coconut") {
        // without coconut feature, we shall just take our mnemonic
        // and derive address from it
        address_sign_cmd = address_sign_cmd.takes_value(true);
    }

    cmd.arg(address_sign_cmd)
}

fn load_identity_keys(pathfinder: &GatewayPathfinder) -> identity::KeyPair {
    let identity_keypair: identity::KeyPair = pemstore::load_keypair(&pemstore::KeyPairPath::new(
        pathfinder.private_identity_key().to_owned(),
        pathfinder.public_identity_key().to_owned(),
    ))
    .expect("Failed to read stored identity key files");
    identity_keypair
}

#[cfg(not(feature = "coconut"))]
fn derive_address(raw_mnemonic: &str) -> AccountId {
    let mnemonic = match raw_mnemonic.parse() {
        Ok(mnemonic) => mnemonic,
        Err(err) => {
            let error_message = format!("failed to parse the provided mnemonic - {}", err).red();
            println!("{}", error_message);
            process::exit(1);
        }
    };
    let wallet =
        match validator_client::nymd::wallet::DirectSecp256k1HdWallet::from_mnemonic(mnemonic) {
            Ok(wallet) => wallet,
            Err(err) => {
                let error_message = format!(
                    "failed to derive your account with the provided mnemonic - {}",
                    err
                )
                .red();
                println!("{}", error_message);
                process::exit(1);
            }
        };
    let account_data = match wallet.try_derive_accounts() {
        Ok(data) => data,
        Err(err) => {
            let error_message = format!(
                "failed to derive your account with the provided mnemonic - {}",
                err
            )
            .red();
            println!("{}", error_message);
            process::exit(1);
        }
    };
    account_data[0].address().clone()
}

#[cfg(not(feature = "coconut"))]
fn sign_derived_address(private_key: &identity::PrivateKey, address: &AccountId) {
    let signature_bytes = private_key.sign(&address.to_bytes()).to_bytes();
    let signature = bs58::encode(signature_bytes).into_string();

    println!(
        "The base58-encoded signature on '{}' is: {}",
        address, signature
    )
}

// we do tiny bit of sanity check validation
#[cfg(feature = "coconut")]
fn sign_provided_address(private_key: &identity::PrivateKey, raw_address: &str) {
    let trimmed = raw_address.trim();

    // try to decode the address (to make sure it's a valid bech32 encoding)
    let (prefix, _) = match bech32::decode(trimmed) {
        Ok(decoded) => decoded,
        Err(err) => {
            let error_message =
                format!("Your wallet address failed to get decoded! Are you sure you copied it correctly?  The error was: {}", err).red();
            println!("{}", error_message);
            process::exit(1);
        }
    };

    if prefix != BECH32_PREFIX {
        let error_message =
            format!("Your wallet address must start with a '{}'", BECH32_PREFIX).red();
        println!("{}", error_message);
        process::exit(1);
    }

    let signature_bytes = private_key.sign(trimmed.as_ref()).to_bytes();
    let signature = bs58::encode(signature_bytes).into_string();

    println!(
        "The base58-encoded signature on '{}' is: {}",
        trimmed, signature
    )
}

// we just sign whatever the user has provided
fn sign_text(private_key: &identity::PrivateKey, text: &str) {
    println!(
        "Signing the text {:?} using your mixnode's Ed25519 identity key...",
        text
    );

    let signature_bytes = private_key.sign(text.as_ref()).to_bytes();
    let signature = bs58::encode(signature_bytes).into_string();

    println!(
        "The base58-encoded signature on '{}' is: {}",
        text, signature
    )
}

pub fn execute(matches: &ArgMatches) {
    let id = matches.value_of(ID_ARG_NAME).unwrap();

    let config = match Config::load_from_file(Some(id)) {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("Failed to load config for {}. Are you sure you have run `init` before? (Error was: {})", id, err);
            return;
        }
    };
    let pathfinder = GatewayPathfinder::new_from_config(&config);
    let identity_keypair = load_identity_keys(&pathfinder);

    if let Some(text) = matches.value_of(SIGN_TEXT_ARG_NAME) {
        sign_text(identity_keypair.private_key(), text)
    }

    #[cfg(not(feature = "coconut"))]
    {
        if matches.is_present(SIGN_ADDRESS_ARG_NAME) {
            let address = derive_address(&config.get_cosmos_mnemonic());
            sign_derived_address(identity_keypair.private_key(), &address);
        }
    }
    #[cfg(feature = "coconut")]
    {
        if let Some(address) = matches.value_of(SIGN_ADDRESS_ARG_NAME) {
            sign_provided_address(identity_keypair.private_key(), address)
        }
    }

    if !matches.is_present(SIGN_TEXT_ARG_NAME) && !matches.is_present(SIGN_ADDRESS_ARG_NAME) {
        let error_message = format!(
            "You must specify either '--{}' or '--{}' argument!",
            SIGN_TEXT_ARG_NAME, SIGN_ADDRESS_ARG_NAME
        )
        .red();
        println!("{}", error_message);
    }
}
