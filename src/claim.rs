use eyre::Result;
use ethers::prelude::{k256::ecdsa::SigningKey, *};
use std::sync::Arc;

use crate::address::{
    AIRDROP_ADDRESS,
    ARBITRUM_CONTRACT,
    NULL_ADDRESS,
};

use crate::helpers::{
    setup_signer,
    address,
    wei_to_float,
};

use crate::timestamp_print;
use colored::*;

#[derive(Debug, Clone)]
pub struct Account {
    address: Address,
    pub eligibility: U256,
    pub http: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
}

impl Account {
    pub async fn new(pkey: String) -> Self {
        let network = std::env::var("NETWORK_RPC").expect("NETWORK_RPC missing");
        let provider = Provider::<Http>::try_from(network).unwrap();
        let (middleware, user_address) = setup_signer(provider.clone(), pkey).await;

        let middleware = Arc::new(middleware);
        let eligibilty = check_eligibility(user_address, middleware.clone()).await;
        Self {
            address: user_address,
            eligibility: eligibilty,
            http: middleware,
        }
    }
    
    pub async fn claim(
        self,
        to: Address,
    ) -> Result<()> {
        abigen!(
            Airdrop,
            r#"[
                function claim() public
            ]"#,
        );
    
        // config provider
        let middleware_claim = self.http.clone();
    
        // setting up airdrop contract middleware
        let contract = Airdrop::new(address(AIRDROP_ADDRESS), middleware_claim);
    
        timestamp_print!(Color::Blue, Some(true), format!("Claiming airdrop with address {:#066x}", self.address));

        loop {
            if let Ok(tx) = contract.claim().send().await {
                if let Some(receipt) = tx.await.unwrap() {
                    if receipt.status.unwrap().as_u64() == 1 {
                        timestamp_print!(Color::Green, Some(true), "Claim succeeded!".to_string());
                        break;
                    } else {
                        timestamp_print!(Color::Red, Some(true), format!("Claim failed with {}!", self.address));
                        timestamp_print!(Color::Blue, Some(false), "Retrying...".to_string());
                    }
                }
            }
        }
    
        if to != address(NULL_ADDRESS) {
            self.send_tokens(to).await?;
        }
        Ok(())
    }
    
    async fn send_tokens(
        self,
        to: Address,
    ) -> Result<()> {
        abigen!(
            ERC20,
            r#"[
                function balanceOf(address) public view returns (uint256)
                function transfer(address,uint256) public
            ]"#,
        );
    
        let token_contract = ERC20::new(address(ARBITRUM_CONTRACT), self.http.clone());
        
        timestamp_print!(
            Color::Blue, Some(true), 
            format!("Sending {} $ARB to {:#066x}", wei_to_float(self.eligibility.as_u128()), to)
        );
        let max_attempt = 15;
        let mut i = 0;

        while i < max_attempt {
            if let Ok(tx) = token_contract.transfer(to, self.eligibility).send().await {
                if let Some(receipt) = tx.await.unwrap() {
                    if receipt.status.unwrap().as_u64() == 1 {
                        timestamp_print!(Color::Green, Some(true), format!("Transfer succeeded with {}!", self.address));
                        break;
                    } else {
                        timestamp_print!(Color::Red, Some(true), "Transfer failed!".to_string());
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            i += 1;
            timestamp_print!(Color::Yellow, Some(true), format!("Attempt {}/{} with {}", i, max_attempt, self.address));
        }

        Ok(())
    }
}

async fn check_eligibility(
    user_address: Address, 
    http: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>
) -> U256 {
    abigen!(
        Airdrop,
        r#"[
            function claimableTokens(address) public view returns (uint256)
        ]"#,
    );

    let contract = Airdrop::new(address(AIRDROP_ADDRESS), http.clone());
    let claimable_tokens = contract.claimable_tokens(user_address.clone()).call().await.unwrap();
    let readable_value = wei_to_float(claimable_tokens.as_u128());

    if readable_value > 0.0 {
        timestamp_print!(Color::Green, Some(true), format!(
            "{:#066x} is eligible to claim {} tokens!", 
            user_address, 
            readable_value
        ));
        claimable_tokens
    } else {
        timestamp_print!(Color::Red, Some(true), format!("{:#066x} is not eligible to claim tokens!", user_address));
        U256::zero()
    }
}