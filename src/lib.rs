pub mod block_scanner;
pub mod helpers;
pub mod claim;
pub mod address;

use std::fs::File;
use std::io::{BufReader, Read};
use serde_json::Value;
use colored::*;

use ethers::prelude::*;

use std::sync::Arc;

use tokio::sync::RwLock;

use tokio::task::JoinHandle;

use crate::block_scanner::loop_blocks;
use crate::claim::Account;
use crate::address::NULL_ADDRESS;
use crate::helpers::address;

use clap::Parser;

#[macro_export]
macro_rules! timestamp_print {
    ($color: expr, $large: expr, $message: expr) => {
        let formatted_message = if let Some(true) = $large {
            $message.chars().map(|c| format!("{}", c).bold().to_string()).collect::<String>()
        } else {
            $message.clone()
        };
        println!(
            "{} {} {}",
            chrono::Local::now()
                .format("[%Y-%m-%d]")
                .to_string()
                .color($color),
            chrono::Local::now()
                .format("[%H:%M:%S]")
                .to_string()
                .color($color),
            formatted_message.color($color)
        );
    };
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Address that you want to send your tokens to
   #[arg(short, long, default_value = NULL_ADDRESS)]
   address: Address,
}

fn get_pkeys_from_file() -> Vec<String> {
    let file = File::open("src/json/addresses.json").expect("Can't open file");
    let mut buf_reader = BufReader::new(file);
    let mut pkeys = Vec::new();

    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).expect("Can't read file");

    let json: Value = serde_json::from_str(&contents).expect("Error parsing JSON");


    if let Value::Array(arr) = json {
        for key in arr {
            if let Value::String(s) = key {
                pkeys.push(s);
            }
        }
    }
    pkeys
}

pub async fn run() {
    let args = Args::parse();
    let pkeys = get_pkeys_from_file();
    
    timestamp_print!(Color::Blue, Some(true), format!("Starting airdrop with {} addresses", pkeys.len()));
    if args.address == address(NULL_ADDRESS) {
        timestamp_print!(Color::Blue, Some(true), "No address provided, tokens will be sent to the address that you used to claim".to_string());
    } else {
        timestamp_print!(Color::Blue, Some(true), format!("Tokens will be sent to {}", args.address));
    }
    timestamp_print!(Color::Blue, Some(false), "Waiting for block 16890400...".to_string());

    let loop_thread_finished = Arc::new(RwLock::new(false));

    
    let loop_thread_finished_clone = Arc::clone(&loop_thread_finished);
    tokio::task::spawn(async move {
        loop_blocks().await;
        *loop_thread_finished_clone.write().await = true;
    });
    
    let mut handles: Vec<JoinHandle<()>> = vec![];

    for pkey in pkeys {
        let address_to = args.address.clone();
        let account = Account::new(pkey).await;
        let account_clone = account.clone();

        let loop_trad_finished_clone = Arc::clone(&loop_thread_finished);

        let handle = tokio::spawn(async move {
            if account_clone.eligibility > U256::zero() {
                while !*loop_trad_finished_clone.read().await {}
                account.claim(address_to).await.unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    timestamp_print!(Color::Green, Some(true), "Claimed all airdrops!".to_string());
}