# Injective Betting Exchange

A decentralized peer-to-peer betting exchange built on the Injective Protocol using CosmWasm smart contracts and a Next.js frontend.

## Table of Contents

1.  [Overview](#overview)
2.  [Smart Contract Logic](#smart-contract-logic)
    * [Core Functionality](#core-functionality)
    * [State Structs](#state-structs)
    * [Execute Messages (Transactions)](#execute-messages-transactions)
    * [Query Messages](#query-messages)
3.  [Local Unit Testing (`cw-multi-test`)](#local-unit-testing-cw-multi-test)
    * [Setup](#setup)
    * [Test Scenarios Covered](#test-scenarios-covered)
4.  [Testnet Deployment (Injective Testnet)](#testnet-deployment-injective-testnet)
    * [Prerequisites](#prerequisites)
    * [Deployment Steps](#deployment-steps)
5.  [Frontend Web Application (Next.js)](#frontend-web-application-nextjs)
    * [Setup](#setup-1)
    * [Key Features Implemented (Conceptual)](#key-features-implemented-conceptual)
    * [Environment Variables](#environment-variables)
6.  [Future Work / Enhancements](#future-work--enhancements)

---

## 1. Overview

This project implements a betting exchange where users can create betting events, place 'back' (betting for an outcome to happen) and 'lay' (betting against an outcome happening) orders. Orders with matching odds, event, and outcome are automatically matched. An oracle resolves events, and funds are distributed to winners. Open (unmatched) orders are refunded upon event resolution or cancellation.

The backend logic is entirely encapsulated within a CosmWasm smart contract deployed on the Injective blockchain. The frontend is a Next.js application allowing users to interact with the deployed smart contract.

---

## 2. Smart Contract Logic

The smart contract (`injective_betting`) manages the creation, matching, and resolution of bets.

### Core Functionality

* **Event Creation:** Users can define new betting markets (events) with a description and assign an oracle.
* **Order Placement:** Users can place 'Back' or 'Lay' orders on active events, specifying their stake, desired odds, and chosen outcome.
    * **Back Order Deposit:** User deposits their `stake`.
    * **Lay Order Deposit:** User deposits their `liability = (odds - 1) * stake`.
* **Order Matching:** When a new order is placed, the contract attempts to match it with existing, compatible counter-orders (same event, outcome, odds, but opposite type) from different users. Matches can be full or partial.
* **Order Cancellation:** Users can cancel their orders if they are not fully matched, and their remaining stake/liability is refunded.
* **Event Resolution:** A designated oracle resolves an event by declaring a winning outcome.
    * **Payouts:** Funds from matched bets (backer's stake + layer's liability) are paid out to the winner (either the backer or the layer, depending on the outcome).
    * **Refunds:** Any remaining open/partially filled orders for the resolved event are cancelled, and stakes/liabilities are refunded.

### State Structs

* **`Config`**: Stores the contract admin and the `betting_denom` (e.g., "uinj").
* **`Event`**: Details of a betting market, including ID, creator, description, oracle, status (Open, Resolved, Cancelled), winning outcome, resolution deadline, and creation time.
* **`Order`**: Details of a specific bet, including ID, event ID, owner, type (Back/Lay), outcome (Yes/No), initial and remaining backer's stake, odds, creation time, and status (Open, PartiallyFilled, Filled, Cancelled).
* **`MatchedBet`**: Records a successful match between a backer and a layer, storing their addresses, the matched stake, the layer's liability, outcome backed, and odds.

### Execute Messages (Transactions)

* **`InstantiateMsg { admin: Option<String>, betting_denom: String }`**: Initializes the contract.
* **`ExecuteMsg::CreateEvent { description: String, oracle_addr: Option<String>, resolution_deadline: Option<Timestamp> }`**: Creates a new betting event.
* **`ExecuteMsg::PlaceOrder { event_id: u64, order_type: OrderType, outcome: Outcome, stake: Uint128, odds: Decimal }`**: Places a new back or lay order. Requires appropriate funds to be sent with the transaction.
* **`ExecuteMsg::CancelOrder { order_id: u64 }`**: Allows the owner to cancel an open/partially filled order.
* **`ExecuteMsg::ResolveEvent { event_id: u64, winning_outcome: Outcome }`**: Allows the designated oracle to resolve an event, triggering payouts and refunds.

### Query Messages

* **`QueryMsg::GetConfig {}`**: Returns the contract configuration.
* **`QueryMsg::GetEvent { event_id: u64 }`**: Returns details for a specific event.
* **`QueryMsg::ListEvents { start_after: Option<u64>, limit: Option<u32>, filter_status: Option<EventStatus> }`**: Lists events with pagination and optional status filtering.
* **`QueryMsg::GetOrder { order_id: u64 }`**: Returns details for a specific order.
* **`QueryMsg::ListOrdersByEvent { event_id: u64, start_after: Option<u64>, limit: Option<u32>, filter_order_type: Option<OrderType>, filter_outcome: Option<Outcome> }`**: Lists open/partially filled orders for a specific event with pagination and filtering.
* **`QueryMsg::ListMatchedBetsByEvent { event_id: u64, start_after: Option<u64>, limit: Option<u32> }`**: Lists matched bets for a specific event with pagination.

---

## 3. Local Unit Testing (`cw-multi-test`)

The smart contract includes a suite of unit tests located in `injective_betting/src/tests.rs`, utilizing the `cw-multi-test` framework for simulating a blockchain environment.

### Setup

The tests use a `default_app()` function to initialize a mock application with predefined user accounts (ADMIN_ID, USER1_ID, USER2_ID, USER3_ID, ORACLE_ID) and initial balances in the `BETTING_DENOM`. A `setup_contract()` helper function handles the instantiation of the betting contract for each test. Address strings for message payloads that require validation (like `InstantiateMsg.admin` or `ExecuteMsg::CreateEvent.oracle_addr`) are generated using `app.api().addr_make("identifier_string").to_string()` to ensure they pass the mock API's bech32 validation.

### Test Scenarios Covered

* **`proper_initialization`**: Verifies that the contract instantiates with the correct admin, betting denomination, and initial ID counters.
* **`create_event`**: Tests the creation of events, both with the sender as the default oracle and with an explicitly specified oracle address.
* **`place_back_order_no_match`**: Checks the placement of a 'Back' order when no corresponding 'Lay' order exists. Verifies order status (`Open`) and that the contract balance reflects the deposited stake.
* **`place_lay_order_no_match`**: Similar to the above, but for a 'Lay' order. Verifies order status and that the contract balance reflects the layer's deposited liability.
* **`place_orders_and_match_full`**: Tests a scenario where a 'Back' order and a 'Lay' order (with identical parameters) are placed and fully match each other. Verifies that both orders become `Filled` and a `MatchedBet` record is created (implicitly by checking contract balance which should sum the stake and liability).
* **`place_orders_and_match_partial`**: Tests a scenario where an incoming order only partially matches an existing larger order. Verifies that one order becomes `PartiallyFilled` with updated remaining stake, and the other becomes `Filled`.
* **`cancel_open_order`**: Checks if a user can cancel their own `Open` order. Verifies that the order status changes to `Cancelled` and the user's funds (stake/liability) are refunded.
* **`resolve_event_backer_wins`**: Tests the event resolution flow where the backer of a matched bet wins. Verifies that the backer receives the correct payout (their stake + layer's liability).
* **`resolve_event_layer_wins`**: Tests event resolution where the layer of a matched bet wins. Verifies that the layer receives the correct payout.
* **`resolve_event_with_open_orders_refund`**: Tests a more complex resolution that includes a matched bet payout (backer wins in this test) and a refund for a separate open (unmatched) order on the same event. Verifies both the payout and the refund by checking user balances and event attributes.

These tests cover the primary lifecycle of events and orders within the betting exchange.

---

## 4. Testnet Deployment (Injective Testnet)

The smart contract has been deployed to the Injective Testnet.

**Deployed Contract Address:** `inj1ym76pag6qww5kpttsczpf280qj4kqcesf5k25s`
**CODE_ID:** `32031`

### Prerequisites

* **`injectived` CLI:** The Injective command-line tool.
* **Funded Testnet Wallet:** A key managed by `injectived` (e.g., named `deployer` with address `inj10pdahsgz3rs5wu4m4rctyutrz8k7m27pnms23x`) must be funded with testnet INJ.
* **Docker:** Required for the `cosmwasm/optimizer` to compile and optimize the Wasm.

### Deployment Steps

1.  **Optimize Wasm:**
    Navigate to the contract's root directory (`injective_betting/`) and run the optimizer:
    ```bash
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      cosmwasm/optimizer:0.16.0 
    ```
    This creates `artifacts/injective_betting.wasm`.

2.  **Configure `injectived`:**
    Set defaults for chain ID, node RPC, gas prices, etc. (Example values, verify current official ones):
    ```bash
    INJECTIVE_TESTNET_CHAIN_ID="injective-888"
    INJECTIVE_TESTNET_NODE_TM_RPC="[https://testnet.sentry.tm.injective.network:443](https://testnet.sentry.tm.injective.network:443)"
    INJECTIVE_TESTNET_GAS_PRICES="160000000inj"

    injectived config set client chain-id "$INJECTIVE_TESTNET_CHAIN_ID"
    injectived config set client node "$INJECTIVE_TESTNET_NODE_TM_RPC"
    injectived config set client broadcast-mode sync 
    # Note: gas-prices and gas-adjustment were specified directly in tx commands
    ```

3.  **Store Wasm Code:**
    ```bash
    injectived tx wasm store artifacts/injective_betting.wasm \
      --from deployer \
      --chain-id "$INJECTIVE_TESTNET_CHAIN_ID" \
      --node "$INJECTIVE_TESTNET_NODE_TM_RPC" \
      --gas auto \
      --gas-adjustment 1.3 \
      --gas-prices "$INJECTIVE_TESTNET_GAS_PRICES" \
      -y
    ```
    Query the resulting `txhash` to find the `CODE_ID`. (Our `CODE_ID` was `32031`).

4.  **Instantiate Contract:**
    ```bash
    CODE_ID="32031" # Use the obtained CODE_ID
    INIT_MSG='{ "admin": "inj10pdahsgz3rs5wu4m4rctyutrz8k7m27pnms23x", "betting_denom": "uinj" }'
    CONTRACT_LABEL="InjectiveBetting_YourUniqueSuffix"
    CONTRACT_INSTANCE_ADMIN_ADDR="inj10pdahsgz3rs5wu4m4rctyutrz8k7m27pnms23x"

    injectived tx wasm instantiate "$CODE_ID" "$INIT_MSG" \
      --label "$CONTRACT_LABEL" \
      --admin "$CONTRACT_INSTANCE_ADMIN_ADDR" \
      --from deployer \
      --chain-id "$INJECTIVE_TESTNET_CHAIN_ID" \
      --node "$INJECTIVE_TESTNET_NODE_TM_RPC" \
      --gas auto \
      --gas-adjustment 1.3 \
      --gas-prices "$INJECTIVE_TESTNET_GAS_PRICES" \
      -y
    ```
    Query the resulting `txhash` to find the `_contract_address`. (Our address is `inj1ym76pag6qww5kpttsczpf280qj4kqcesf5k25s`).

---

## 5. Frontend Web Application (Next.js)

A Next.js application (`ui/` directory) provides the user interface for interacting with the deployed smart contract.

### Setup

1.  Navigate to the `ui/` directory.
2.  Install dependencies: `npm install` (or `yarn install`).
3.  Ensure you have a `.env.local` file in the `ui/` directory with the following (replace with your actual contract address):
    ```env
    NEXT_PUBLIC_INJECTIVE_CHAIN_ID="injective-888"
    NEXT_PUBLIC_CONTRACT_ADDRESS="inj1ym76pag6qww5kpttsczpf280qj4kqcesf5k25s"
    # NEXT_PUBLIC_NETWORK_NAME="testnet" (optional, defaults to testnet in code)
    ```
4.  Run the development server: `npm run dev` (or `yarn dev`).

### Key Features Implemented (Conceptual Foundation)

The frontend uses the Injective TypeScript SDK (`@injectivelabs/sdk-ts`, `@injectivelabs/wallet-ts`) to communicate with the Injective testnet.

* **Wallet Connection (`WalletContext.tsx`, `ConnectWalletButton.tsx`):**
    * Allows users to connect their Keplr wallet.
    * Manages connection state (address, signer) globally using React Context.
    * Uses `window.keplr` directly to enable the chain and get an `OfflineSigner`.
* **SDK Client Initialization (`lib/network.ts`):**
    * Sets up gRPC and REST clients for interacting with the Injective network using endpoints from `@injectivelabs/networks`.
* **Type Definitions (`lib/types.ts`):**
    * TypeScript interfaces mirroring Rust structs for messages, state, and query responses, ensuring type safety.
* **Contract Interaction Layer (`lib/contractInteractions.ts`):**
    * **Query Functions:**
        * `queryListEvents`: Fetches a list of events. Uses `chainGrpcWasmApi.fetchSmartContractState` and `Buffer` for base64 encoding/decoding of query messages and responses.
    * **Execute Functions:**
        * `executeCreateEvent`: Constructs `MsgExecuteContractCompat`, uses `WalletStrategy` (configured for Keplr) and `MsgBroadcaster` to send transactions.
        * `executePlaceOrder` (conceptual): Similar structure for placing orders, handling `funds` correctly (converting to base denomination string for the SDK).
* **UI Components (`components/`):**
    * `EventList.tsx`: Displays fetched events.
    * `CreateEventForm.tsx`: A form for users to create new betting events.
    * `PlaceOrderForm.tsx` (conceptual): A form for users to place bets on an event.
* **Main Page (`app/page.tsx`):**
    * Integrates the wallet connection, event creation form, and event list display.
    * Handles fetching initial data and refreshing data after transactions.

### Environment Variables

* `NEXT_PUBLIC_INJECTIVE_CHAIN_ID`: The chain ID for the Injective network (e.g., "injective-888" for testnet).
* `NEXT_PUBLIC_CONTRACT_ADDRESS`: The address of the deployed betting exchange smart contract.
* `NEXT_PUBLIC_NETWORK_NAME` (Optional): Can be set to "testnet" or "mainnet" to configure SDK network endpoints in `lib/network.ts`.

---

## 6. Future Work / Enhancements

* Complete implementation of all contract interactions in the frontend (place order, cancel, resolve, view order details, view matched bets).
* Robust error handling and user feedback in the UI.
* Comprehensive styling and improved UX.
* Detailed event detail pages.
* User-specific views (e.g., "My Orders", "My Bets").
* Real-time updates (e.g., using WebSockets or polling for new orders/matches).
* Implementation of a secure and reliable oracle mechanism for event resolution on testnet/mainnet.
* More extensive unit and end-to-end testing for the frontend.
