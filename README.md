# Crowdfunding on NEAR

## Overview
This project is a decentralized crowdfunding platform built on the NEAR blockchain. It allows users to create campaigns, contribute funds, and withdraw collected funds upon successful completion of the campaign. The project consists of:
- A **Rust-based smart contract** deployed on NEAR
- A **Next.js frontend** to interact with the contract

## Project Structure
```
Crowdfunding-NEAR/
│── frontend/              # Next.js frontend application
│── smart-contracts/       # Rust smart contracts for NEAR blockchain
│── .gitignore             # Git ignore file
│── README.md              # Project documentation
```

## Features
- **Create Campaigns**: Users can create campaigns by setting a name, funding goal, and duration.
- **Contribute to Campaigns**: Users can contribute NEAR tokens to active campaigns.
- **Withdraw Funds**: Campaign owners can withdraw funds after successful funding.
- **Automatic Finalization**: Campaigns automatically finalize when the deadline passes.
- **Refund System**: If a campaign fails, contributors can request refunds.

## Smart Contract Deployment
The smart contract is written in Rust and deployed on the NEAR testnet.

### Build and Deploy
To deploy the contract, navigate to the `smart-contracts` directory and run:
```sh
cargo build --target wasm32-unknown-unknown --release
near deploy <your-account>.testnet ./target/wasm32-unknown-unknown/release/campaign.wasm
```

### Interacting with the Contract
#### Create a Campaign
```sh
near call <your-account>.testnet create_campaign '{"name": "Test Campaign", "funding_goal": "1000000000000000000000000", "duration_seconds": 86400}' --accountId <your-account>.testnet --gas 300000000000000 --deposit 0
```

#### View All Campaigns
```sh
near view <your-account>.testnet get_all_campaigns '{}'
```

#### Contribute to a Campaign
```sh
near call <your-account>.testnet contribute '{"campaign_id": 0}' --accountId <your-account>.testnet --deposit 10
```

## Frontend Setup
The frontend is built using Next.js and interacts with the smart contract via the NEAR Wallet Selector.

### Install Dependencies
```sh
cd frontend
npm install
```

### Start the Development Server
```sh
npm run dev
```

### Build for Production
```sh
npm run build
npm start
```

## Configuration
Update `config.js` to specify the correct contract ID for testnet or mainnet:
```js
const contractPerNetwork = {
  mainnet: "hello.near-examples.near",
  testnet: "your-account.testnet",
};
export const NetworkId = "testnet";
export const HelloNearContract = contractPerNetwork[NetworkId];
```

## Git Best Practices
To avoid committing unnecessary files, ensure `.gitignore` includes:
```
node_modules/
frontend/.next/
frontend/out/
smart-contracts/target/
.DS_Store
```

## License
This project is open-source and available under the MIT License.

---
**Author:** Matt Peters

