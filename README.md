<a name="readme-top"></a>

<h3 align="center">Blockheaders/Transactions Tool</h3>

  <p align="center">
    Updates/Fixes gaps in blockheaders DB. Data is queried via RPC calls

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->

## Getting Started

To get this application runninng locally, follow these steps.

### Prerequisites

What you would need:

- Rust
  ```
  https://www.rust-lang.org/tools/install
  ```

### Installation

1. Replace the placeholder **database connection string** in _src/db.rs_ with the actual connection string.
1. Replace the placeholder **rpc endpoint string** in _src/endpoints.rs_ with the actual RPC endpoint string.
1. Clone the repo
   ```sh
   git clone https://github.com/jonathantxj/BlockheadersUpdater.git
   ```
1. Build project
   ```sh
   cargo build
   ```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->

## Usage

This application has 2 features that are similar but with different purposes.

### Mode 1 - Update

Fetches blockheaders and transaction data via RPC and writes to DB

**Usage:** _cargo run update_

**Optional parameters:**

1. _start <block_number>_

- First block number to start updating the database from. (Inclusive)
- **Default**: Latest block in the database + 1

2. _end <block_number>_

- Last block number to update the database to. (Inclusive)
- **Default**: Latest block in ethereum chain at the time of running the command

**Examples:**

```sh
cargo run update
```

```sh
cargo run update --start 19983846
```

```sh
cargo run update --end 19983849
```

```sh
cargo run update --start 19983846 --end 19983849
```

### Mode 2 - Fix

Patches missing blockheaders and transaction data from the DB, retrieving via RPC

**Usage:** _cargo run update_

**Optional parameters:**

1. _start <block_number>_

- First block number to start checking the database from. (Inclusive)
- **Default**: 0

2. _end <block_number>_

- Last block number to check the database to. (Inclusive)
- **Default**: Last entry in the database

**Examples:**

```sh
cargo run fix
```

```sh
cargo run fix --start 19983846
```

```sh
cargo run fix --end 19983849
```

```sh
cargo run fix --start 19983846 --end 19983849
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>
