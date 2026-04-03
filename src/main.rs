use std::env;

fn load_dotenv() {
    let env_path = format!("{}/.env", env!("CARGO_MANIFEST_DIR"));
    if dotenv::from_path(&env_path).is_ok() {
        return;
    }
    let _ = dotenv::dotenv();
}

#[tokio::main]
async fn main() {
    load_dotenv();

    let standard_hello_message = String::from("🦄 Welcome to the Uniswap MCP Server");
    print_hello_message(&String::from("Jean"));
    println!("{}", standard_hello_message);

    match get_quote().await {
        Ok(quote) => println!("Quote: {:?}", quote),
        Err(e) => println!("Error: {}", e),
    }
}

fn print_hello_message(username: &String) {
    println!("Hello {}!", username);
}

// Endpoint to get a quote
// TODO: this should be called `quote` as it is an endpoint
async fn get_quote() -> Result<serde_json::Value, String> {
    let api_key = env::var("UNISWAP_API_KEY").map_err(|_| {
        "UNISWAP_API_KEY is not set (export it or add it to .env in this crate or the repo root)"
    })?;

    let request_url = String::from("https://trade-api.gateway.uniswap.org/v1/quote");

    // Make not blocking so that MCP server can handle multiple requests at once from multiple 👤 users / 🤖 AI agents.
    let client = reqwest::Client::new();

    // Construct the JSON body as per the example of Uniswap's API docs
    let json_body = serde_json::json!({
        "tokenIn": "0x0000000000000000000000000000000000000000", // ETH
        "tokenOut": "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
        "tokenInChainId": 1,
        "tokenOutChainId": 1,
        "type": "EXACT_INPUT",
        "amount": "1000000000000000000", // 1 ETH in wei
        "swapper": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", // User's wallet address (vitalik.eth)
        "slippageTolerance": 0.5 // 0.5%
    });

    let response = client
        .post(&request_url)
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&json_body)
        .send()
        .await
        .map_err(|e| format!("Failed to send: {}", e))?;

    let quote_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse quote response JSON: {}", e))?;

    Ok(quote_data)
}
