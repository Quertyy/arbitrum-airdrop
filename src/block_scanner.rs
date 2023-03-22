use ethers::prelude::*;

use crate::timestamp_print;
use colored::*;

pub async fn loop_blocks() {
    let ws_network = std::env::var("NETWORK_WSS").expect("missing NETWORK_WSS");
    let ws_provider: Provider<Ws> = Provider::<Ws>::connect(ws_network).await.unwrap();

    let mut last_block: U64 = U64::zero();
    let mut stream = ws_provider.subscribe_blocks().await.unwrap();

    while let Some(block) = stream.next().await {
        if block.number.unwrap() < U64::from(16890400) {
            let block_number = block.number.unwrap();
            if block_number > last_block {
                last_block = block_number;
                timestamp_print!(
                    Color::White,
                    Some(false),
                    format!("BLOCK: {} | BLOCKS LEFT: {}", block_number, 16890400 - block_number.as_u64())
                );
            }
        } else {
            timestamp_print!(
                Color::Green,
                Some(true),
                format!("BLOCK: {} | CLAIMING STARTED", block.number.unwrap())
            );
            break;
        }
    }
    
}