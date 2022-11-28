# Data Dictionary

_Oura_ follows a Cardano chain and outputs events. Each event contains data about itself and about the _context_ in which it occurred.

A consumer aggregating a sequence of multiple events will notice redundant / duplicated data. For example, the "block number" value will appear repeated in the context of every event of the same block. This behavior is intended, making each event a self-contained record is an architectural decision. We favor "consumption ergonomics" over "data normalization".

## Available Events

The following list represent the already implemented events. These data structures are represented as an `enum` at the code level.

### `RollBack` Event

Data on chain rollback(The result of the local node switching to the consensus chains).

| Name         | DataType   | Description                    |
| :---         | :---       | :---                           |
| block_slot   | u64        | Slot of the rolled back block. |
| block_hash   | String     | Hash of the rolled back block. |


<br />
<br />
<hr />


### `Block` Event

Data on an issued block.

| Name         | DataType   | Description                           |
| :---         | :---       | :---                                  |
| body_size    | usize      | Size of the block.                    |
| issuer_vkey  | String     | Block issuer Public verification key. |

**Context**

| Name         | DataType     | Description                           |
| :---         | :---         | :---                                  |
| block_number | Option\<u64> | Height of block from genesis.         |
| slot         | Option\<u64> | Current slot.                         |


<br />
<br />
<hr />


### `Transaction` Event

Data on a transaction.

| Name                    | DataType    | Description                            |
| :---                    | :---        | :---                                   |
| fee                     | u64         | Transaction fees in lovelace.          |
| ttl                     | Option\<u64> | Transaction time to live.              |
| validity_interval_start | Option\<u64> | Start of transaction validity interval |
| network_id              | Option\<u32> | Network ID.                            |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `TxInput` Event

Data on a transaction input.

| Name         | DataType     | Description                           |
| :---         | :---         | :---                                  |
| tx_id        | String       | Transaction ID.                       |
| index        | u64          | Index of input in transaction inputs. |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |
| input_idx    | Option\<usize>  | Input ID.                             |


<br />
<br />
<hr />


### `TxOutput` Event

Data on a transaction output (UTXO).

| Name         | DataType     | Description                           |
| :---         | :---         | :---                                  |
| address      | String       | Address of UTXO.                      |
| amount       | u64          | Amount of lovelace in UTXO.           |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |
| output_idx   | Option\<usize>  | Output ID.                            |


<br />
<br />
<hr />


### `OutputAsset` Event

Data on a non-ADA asset in a UTXO.

| Name         | DataType     | Description                           |
| :---         | :---         | :---                                  |
| policy       | String       | Minting policy of asset.              |
| asset        | String       | Asset ID.                             |
| amount       | u64          | Amount of asset.                      |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |
| output_idx   | Option\<usize>  | Output ID.                            |


<br />
<br />
<hr />


### `Metadata` Event

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| key          | String          | ....                                  |
| subkey       | Option\<String> | ....                                  |
| value        | Option\<String> | ....                                  |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `Mint` Event

Data on the minting of a non-ADA asset.

| Name         | DataType     | Description                           |
| :---         | :---         | :---                                  |
| policy       | String       | Minting policy of asset.              |
| asset        | String       | Asset ID.                             |
| quantity     | i64          | Quantity of asset minted.             |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `Collateral` Event

Data on [collateral inputs](https://docs.cardano.org/plutus/collateral-mechanism).

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| tx_id        | String          | Transaction ID.                       |
| index        | u64             | Index of transaction input in inputs. |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `PlutusScriptRef` Event

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| data         | String          | ....                                  |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `StakeRegistration` Event

Data on stake registration event.

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| credential   | [StakeCredential](https://docs.rs/oura/1.4.1/oura/model/enum.StakeCredential.html) | Staking credentials.                 |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `StakeDeregistration` Event

Data on stake deregistration event.

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| credential   | [StakeCredential](https://docs.rs/oura/1.4.1/oura/model/enum.StakeCredential.html) | Staking credentials.                 |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `StakeDelegation` Event

Data on [stake delegation](https://docs.cardano.org/core-concepts/delegation) event.

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| credential   | [StakeCredential](https://docs.rs/oura/1.4.1/oura/model/enum.StakeCredential.html) | Stake credentials.                    |
| pool_hash    | String          | Hash of stake pool ID.                |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `PoolRegistration` Event

Data on the stake [registration event](https://developers.cardano.org/docs/stake-pool-course/handbook/register-stake-pool-metadata/).

| Name           | DataType        | Description                            |
| :---           | :---            | :---                                   |
| operator       | String          | Stake pool operator ID.                |
| vrf_keyhash    | String          | Kehash of node VRF operational key.    |
| pledge         | u64             | Stake pool pledge (lovelace).          |
| cost           | u64             | Operational costs per epoch (lovelace).|
| margin         | f64             | Operator margin.                       |
| reward_account | String          | Account to receive stake pool rewards. |
| pool_owners    | Vec\<String>    | Stake pool owners.                     |
| relays         | Vec\<String>    | ....                                   |
| pool_metadata  | Option\<String> | ....                                   |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `PoolRetirement` Event

Data on [stake pool retirement](https://cardano-foundation.gitbook.io/stake-pool-course/stake-pool-guide/stake-pool/retire_stakepool) event.

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| pool         | String          | Pool ID.                              |
| epoch        | u64             | Current epoch.                        |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `GenesisKeyDelegation` Event

Data on genesis key delegation.

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Current slot.                         |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |


<br />
<br />
<hr />


### `MoveInstantaneousRewardsCert` Event

| Name                 | DataType                                | Description                           |
| :---                 | :---                                    | :---                                  |
| from_reserves        | bool                                    | ....                                  |
| from_treasury        | bool                                    | ....                                  |
| to_stake_credentials | Option\<BTreeMap<StakeCredential, i64>> | ....                                  |
| to_other_pot         | Option\<u64>                            | ....                                  |

**Context**

| Name         | DataType        | Description                           |
| :---         | :---            | :---                                  |
| block_number | Option\<u64>    | Height of block from genesis.         |
| slot         | Option\<u64>    | Blockchain slot.                      |
| tx_idx       | Option\<usize>  | Transaction Index.                       |
| tx_hash      | Option\<String> | Transaction hash.                     |
