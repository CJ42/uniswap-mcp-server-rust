# 🦄 Uniswap MCP Server Rust 🦀

![Cover image](./cover.png)

An MCP (Model Context Protocol) server in Rust that exposes **Uniswap Trade API** quotes as a tool for AI clients. Built with the official Rust SDK **[RMCP](https://github.com/modelcontextprotocol/rust-sdk)** (`rmcp` on crates.io).

## Prerequisites

- Rust toolchain (`cargo`, `rustc`)
- A Uniswap **Trade API** key as `UNISWAP_API_KEY` (see [Uniswap docs](https://docs.uniswap.org/))

## Configuration

Create a `.env` file next to `Cargo.toml` (or export the variable in your shell):

```bash
UNISWAP_API_KEY=your_key_here
```

The server loads `.env` at startup using the crate manifest directory, so it works when MCP hosts run the binary from another working directory.

## Build

```bash
cargo build --release
```

## How it works

- **Transport:** stdio (standard for local MCP: the host spawns this process and speaks JSON-RPC over stdin/stdout).
- **SDK:** [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) with `server` + `transport-io`.
- **Tool:** `uniswap_swap_quote` — calls `POST https://trade-api.gateway.uniswap.org/v1/quote` with your API key and returns the JSON quote (pretty-printed text). Use **`token_in_preset`** to pick the asset you sell on Ethereum mainnet without typing addresses: `native_eth`, `usdc`, `usdt`, `wbtc`, `dai`, `weth`. If you set a preset, it overrides `token_in`; otherwise pass a full `token_in` address.

Async runtime: **Tokio** (required by RMCP and `reqwest`).

## Using with an MCP client

Point your MCP host at the compiled binary. Example for **Cursor** (`.cursor/mcp.json` or global MCP settings):

```json
{
  "mcpServers": {
    "uniswap": {
      "command": "/absolute/path/to/uniswap-api-rust/target/release/uniswap-mcp-server-rust",
      "env": {
        "UNISWAP_API_KEY": "your_key_here"
      }
    }
  }
}
```

You can omit `env` if `UNISWAP_API_KEY` is already in the environment.

To debug with the official inspector (see RMCP examples):

```bash
npx @modelcontextprotocol/inspector /absolute/path/to/target/debug/uniswap-mcp-server-rust
```

## Example to test

Run the commands from the previous section and open the MCP Inspector in your browser.

1. Enter the following values as example

```
"tokenIn": "0x0000000000000000000000000000000000000000", // ETH
"tokenOut": "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
"tokenInChainId": 1,
"tokenOutChainId": 1,
"type": "EXACT_INPUT",
"amount": "1000000000000000000", // 1 ETH in wei
"swapper": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", // User's wallet address (vitalik.eth)
```

2. Click on the "Run Tool" button

## Learning steps

- [x] MCP server over stdio with RMCP
- [x] Async quote tool via Uniswap Trade API
- [ ] Make it I can enter different commands while it is running 
(`start`, `stop`, `unicorns`)
- [ ] Try to use functionalities from the [**allow** web3 library for 
Rust.](https://github.com/alloy-rs)
- [ ] Extend with more tools (e.g. supported chains, token metadata)

## License

See `LICENSE` in the repository.
