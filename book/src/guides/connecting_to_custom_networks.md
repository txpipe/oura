# Connecting to custom networks

This guide shows you how to configure Oura to connect to a custom (not mainnet or testnet) network.


## Instructions

### 1. Add Network specific configurations

Add the following data on the network to config TOML file.

```TOML
[chain]
byron_epoch_length  = u32
byron_slot_length = u32
byron_known_slot = u64
byron_known_hash = String
byron_known_time = u64
shelley_epoch_length = u32
shelley_slot_length = u32
shelley_known_slot = u64
shelley_known_hash = String
shelley_known_time = u64
address_hrp = String
adahandle_policy = String
```

Some details on the cofigurataion:

| Name                 | DataType | Description                                 |
| :------------------- | :------- | :------------------------------------------ |
| byron_epoch_length   | u32      | ....                                        |
| byron_slot_length    | u32      | ....                                        |
| byron_known_slot     | u32      | ....                                        |
| byron_known_hash     | String   | ....                                        |
| byron_known_time     | u64      | ....                                        |
| shelley_epoch_length | u32      | ....                                        |
| shelley_slot_length  | u32      | ....                                        |
| shelley_known_slot   | u32      | ....                                        |
| shelley_known_hash   | String   | ....                                        |
| shelley_known_time   | u64      | ....                                        |
| address_hrp          | String   | ....                                        |
| adahandle_policy     | String   | Minting policy of AdaHandle on the network. |
