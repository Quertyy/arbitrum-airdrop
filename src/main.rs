use arbitrum_airdrop::run;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    run().await;
}
