use std::env;

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};

fn load_dotenv() {
    let env_path = format!("{}/.env", env!("CARGO_MANIFEST_DIR"));
    if dotenv::from_path(&env_path).is_ok() {
        return;
    }
    let _ = dotenv::dotenv();
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SwapQuoteArgs {
    #[schemars(
        description = "ERC-20 token address to sell (use 0x000…0000 for native ETH on the chain)"
    )]
    token_in: String,
    #[schemars(description = "ERC-20 token address to buy")]
    token_out: String,
    #[serde(default = "default_chain_id")]
    #[schemars(description = "Chain ID for token_in (e.g. 1 for Ethereum mainnet)")]
    token_in_chain_id: u64,
    #[serde(default = "default_chain_id")]
    #[schemars(description = "Chain ID for token_out")]
    token_out_chain_id: u64,
    #[serde(default = "default_swap_type")]
    #[schemars(description = "Quote type, usually EXACT_INPUT")]
    r#type: String,
    #[schemars(description = "Input amount in the token's smallest unit (e.g. wei for ETH)")]
    amount: String,
    #[schemars(description = "Wallet address that will execute the swap (swapper)")]
    swapper: String,
    #[serde(default = "default_slippage")]
    #[schemars(description = "Max slippage tolerance as a fraction (e.g. 0.5 for 0.5%)")]
    slippage_tolerance: f64,
}

fn default_chain_id() -> u64 {
    1
}

fn default_swap_type() -> String {
    "EXACT_INPUT".into()
}

fn default_slippage() -> f64 {
    0.5
}

#[derive(Clone)]
struct UniswapServer {
    tool_router: ToolRouter<Self>,
    http: reqwest::Client,
}

#[tool_router]
impl UniswapServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            http: reqwest::Client::new(),
        }
    }

    #[tool(
        description = "Fetch a swap quote from the Uniswap Trade API (gateway). Requires UNISWAP_API_KEY in the environment or .env file."
    )]
    async fn uniswap_swap_quote(
        &self,
        Parameters(args): Parameters<SwapQuoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        let api_key = env::var("UNISWAP_API_KEY").map_err(|_| {
            McpError::invalid_params(
                "UNISWAP_API_KEY is not set; add it to .env or export it before starting the server.",
                None,
            )
        })?;

        let body = serde_json::json!({
            "tokenIn": args.token_in,
            "tokenOut": args.token_out,
            "tokenInChainId": args.token_in_chain_id,
            "tokenOutChainId": args.token_out_chain_id,
            "type": args.r#type,
            "amount": args.amount,
            "swapper": args.swapper,
            "slippageTolerance": args.slippage_tolerance,
        });

        let response = self
            .http
            .post("https://trade-api.gateway.uniswap.org/v1/quote")
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("HTTP request failed: {e}"), None))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read body: {e}"), None))?;

        if !status.is_success() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Uniswap API returned {status}: {text}"
            ))]));
        }

        let pretty = serde_json::from_str::<serde_json::Value>(&text)
            .map(|v| serde_json::to_string_pretty(&v).unwrap_or(text.clone()))
            .unwrap_or(text);

        Ok(CallToolResult::success(vec![Content::text(pretty)]))
    }
}

#[tool_handler]
impl ServerHandler for UniswapServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "Tools: uniswap_swap_quote — get a DEX quote from Uniswap's Trade API. Set UNISWAP_API_KEY.".to_string(),
            )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_dotenv();

    let service = UniswapServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("MCP server error: {e}"))?;

    let quit = service.waiting().await?;
    eprintln!("Server stopped: {quit:?}");
    Ok(())
}
