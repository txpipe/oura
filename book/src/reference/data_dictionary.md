# Data Dictionary

_Oura_ follows a Cardano chain and outputs events. Each event contains data about itself and about the _context_ in which it occurred.

A consumer aggregating a sequence of multiple events will notice redundant / duplicated data. For example, the "block number" value will appear repeated in the context of every event of the same block. This behavior is intended, making each event a self-contained record is an architectural decision. We favor "consumption ergonomics" over "data normalization".

## Available Events

The following list represent the already implemented events. These data structures are represented as an `enum` at the code level.

### `RollBack` Event

- block_slot: u64,
- block_hash: String,

### `Block` Event

- body_size: usize,
- issuer_vkey: String,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,

### `Transaction` Event

- fee: u64,
- ttl: Option<u64>,
- validity_interval_start: Option<u64>,
- network_id: Option<u32>,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `TxInput` Event

- tx_id: String,
- index: u64,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,
    - input_idx: Option<usize>,

### `TxOutput` Event

- address: String,
- amount: u64,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,
    - output_idx: Option<usize>,

### `OutputAsset` Event

- policy: String,
- asset: String,
- amount: u64,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,
    - output_idx: Option<usize>,

### `Metadata` Event

- key: String,
- subkey: Option<String>,
- value: Option<String>,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `Mint` Event

- policy: String,
- asset: String,
- quantity: i64,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `Collateral` Event

- tx_id: String,
- index: u64,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `PlutusScriptRef` Event

- data: String
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `StakeRegistration` Event

- credential: StakeCredential
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `StakeDeregistration` Event

- credential: StakeCredential
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `StakeDelegation` Event

- credential: StakeCredential,
- pool_hash: String,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `PoolRegistration` Event

- operator: String,
- vrf_keyhash: String,
- pledge: u64,
- cost: u64,
- margin: f64,
- reward_account: String,
- pool_owners: Vec<String>,
- relays: Vec<String>,
- pool_metadata: Option<String>,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `PoolRetirement` Event

- pool: String,
- epoch: u64,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `GenesisKeyDelegation` Event
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,

### `MoveInstantaneousRewardsCert` Event

- from_reserves: bool,
- from_treasury: bool,
- to_stake_credentials: Option<BTreeMap<StakeCredential, i64>>,
- to_other_pot: Option<u64>,
- context:
    - block_number: Option<u64>,
    - slot: Option<u64>,
    - tx_idx: Option<usize>,
    - tx_hash: Option<String>,
