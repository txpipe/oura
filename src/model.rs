use std::fmt::Display;

use merge::Merge;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use strum_macros::Display;

// We're duplicate the Era struct from Pallas for two reasons: a) we need it to
// be serializable and we don't want to impose serde dependency on Pallas and b)
// we prefer not to add dependencies to Pallas outside of the sources that
// actually use it on an attempt to make the pipeline agnostic of particular
// implementation details.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Display)]
pub enum Era {
    Undefined,
    Byron,
    Shelley,
    Allegra,
    Mary,
    Alonzo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetadatumRendition {
    MapJson(JsonValue),
    ArrayJson(JsonValue),
    IntScalar(i128),
    TextScalar(String),
    BytesHex(String),
}

impl Display for MetadatumRendition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadatumRendition::MapJson(x) => x.fmt(f),
            MetadatumRendition::ArrayJson(x) => x.fmt(f),
            MetadatumRendition::IntScalar(x) => x.fmt(f),
            MetadatumRendition::TextScalar(x) => x.fmt(f),
            MetadatumRendition::BytesHex(x) => x.fmt(f),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MetadataRecord {
    pub label: String,

    #[serde(flatten)]
    pub content: MetadatumRendition,
}

impl From<MetadataRecord> for EventData {
    fn from(x: MetadataRecord) -> Self {
        EventData::Metadata(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CIP25AssetRecord {
    pub version: String,
    pub policy: String,
    pub asset: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub media_type: Option<String>,
    pub description: Option<String>,
    pub raw_json: JsonValue,
}

impl From<CIP25AssetRecord> for EventData {
    fn from(x: CIP25AssetRecord) -> Self {
        EventData::CIP25Asset(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CIP15AssetRecord {
    pub voting_key: String,
    pub stake_pub: String,
    pub reward_address: String,
    pub nonce: i64,
    pub raw_json: JsonValue,
}

impl From<CIP15AssetRecord> for EventData {
    fn from(x: CIP15AssetRecord) -> Self {
        EventData::CIP15Asset(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TxInputRecord {
    pub tx_id: String,
    pub index: u64,
}

impl From<TxInputRecord> for EventData {
    fn from(x: TxInputRecord) -> Self {
        EventData::TxInput(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutputAssetRecord {
    pub policy: String,
    pub asset: String,
    pub asset_ascii: Option<String>,
    pub amount: u64,
}

impl From<OutputAssetRecord> for EventData {
    fn from(x: OutputAssetRecord) -> Self {
        EventData::OutputAsset(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TxOutputRecord {
    pub address: String,
    pub amount: u64,
    pub assets: Option<Vec<OutputAssetRecord>>,
    pub datum_hash: Option<String>,
}

impl From<TxOutputRecord> for EventData {
    fn from(x: TxOutputRecord) -> Self {
        EventData::TxOutput(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MintRecord {
    pub policy: String,
    pub asset: String,
    pub quantity: i64,
}

impl From<MintRecord> for EventData {
    fn from(x: MintRecord) -> Self {
        EventData::Mint(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionRecord {
    pub hash: String,
    pub fee: u64,
    pub ttl: Option<u64>,
    pub validity_interval_start: Option<u64>,
    pub network_id: Option<u32>,
    pub input_count: usize,
    pub output_count: usize,
    pub mint_count: usize,
    pub total_output: u64,

    // include_details
    pub metadata: Option<Vec<MetadataRecord>>,
    pub inputs: Option<Vec<TxInputRecord>>,
    pub outputs: Option<Vec<TxOutputRecord>>,
    pub mint: Option<Vec<MintRecord>>,
    pub vkey_witnesses: Option<Vec<VKeyWitnessRecord>>,
    pub native_witnesses: Option<Vec<NativeWitnessRecord>>,
    pub plutus_witnesses: Option<Vec<PlutusWitnessRecord>>,
    pub plutus_redeemers: Option<Vec<PlutusRedeemerRecord>>,
    pub plutus_data: Option<Vec<PlutusDatumRecord>>,
}

impl From<TransactionRecord> for EventData {
    fn from(x: TransactionRecord) -> Self {
        EventData::Transaction(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Merge, Default)]
pub struct EventContext {
    pub block_hash: Option<String>,
    pub block_number: Option<u64>,
    pub slot: Option<u64>,
    pub timestamp: Option<u64>,
    pub tx_idx: Option<usize>,
    pub tx_hash: Option<String>,
    pub input_idx: Option<usize>,
    pub output_idx: Option<usize>,
    pub output_address: Option<String>,
    pub certificate_idx: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StakeCredential {
    AddrKeyhash(String),
    Scripthash(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VKeyWitnessRecord {
    pub vkey_hex: String,
    pub signature_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NativeWitnessRecord {
    pub policy_id: String,
    pub script_json: JsonValue,
}

impl From<NativeWitnessRecord> for EventData {
    fn from(x: NativeWitnessRecord) -> Self {
        EventData::NativeWitness(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlutusWitnessRecord {
    pub script_hash: String,
    pub script_hex: String,
}

impl From<PlutusWitnessRecord> for EventData {
    fn from(x: PlutusWitnessRecord) -> Self {
        EventData::PlutusWitness(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlutusRedeemerRecord {
    pub purpose: String,
    pub ex_units_mem: u32,
    pub ex_units_steps: u64,
    pub input_idx: u32,
    pub plutus_data: JsonValue,
}

impl From<PlutusRedeemerRecord> for EventData {
    fn from(x: PlutusRedeemerRecord) -> Self {
        EventData::PlutusRedeemer(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlutusDatumRecord {
    pub datum_hash: String,
    pub plutus_data: JsonValue,
}

impl From<PlutusDatumRecord> for EventData {
    fn from(x: PlutusDatumRecord) -> Self {
        EventData::PlutusDatum(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockRecord {
    pub era: Era,
    pub epoch: Option<u64>,
    pub epoch_slot: Option<u64>,
    pub body_size: usize,
    pub issuer_vkey: String,
    pub tx_count: usize,
    pub slot: u64,
    pub hash: String,
    pub number: u64,
    pub previous_hash: String,
    pub cbor_hex: Option<String>,
    pub transactions: Option<Vec<TransactionRecord>>,
}

impl From<BlockRecord> for EventData {
    fn from(x: BlockRecord) -> Self {
        EventData::Block(x)
    }
}

#[derive(Serialize, Deserialize, Display, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EventData {
    Block(BlockRecord),
    BlockEnd(BlockRecord),
    Transaction(TransactionRecord),
    TransactionEnd(TransactionRecord),
    TxInput(TxInputRecord),
    TxOutput(TxOutputRecord),
    OutputAsset(OutputAssetRecord),
    Metadata(MetadataRecord),

    VKeyWitness(VKeyWitnessRecord),
    NativeWitness(NativeWitnessRecord),
    PlutusWitness(PlutusWitnessRecord),
    PlutusRedeemer(PlutusRedeemerRecord),
    PlutusDatum(PlutusDatumRecord),

    #[serde(rename = "cip25_asset")]
    CIP25Asset(CIP25AssetRecord),

    #[serde(rename = "cip15_asset")]
    CIP15Asset(CIP15AssetRecord),

    Mint(MintRecord),
    Collateral {
        tx_id: String,
        index: u64,
    },
    NativeScript {
        policy_id: String,
        script: JsonValue,
    },
    PlutusScript {
        hash: String,
        data: String,
    },
    StakeRegistration {
        credential: StakeCredential,
    },
    StakeDeregistration {
        credential: StakeCredential,
    },
    StakeDelegation {
        credential: StakeCredential,
        pool_hash: String,
    },
    PoolRegistration {
        operator: String,
        vrf_keyhash: String,
        pledge: u64,
        cost: u64,
        margin: f64,
        reward_account: String,
        pool_owners: Vec<String>,
        relays: Vec<String>,
        pool_metadata: Option<String>,
    },
    PoolRetirement {
        pool: String,
        epoch: u64,
    },
    GenesisKeyDelegation,
    MoveInstantaneousRewardsCert {
        from_reserves: bool,
        from_treasury: bool,
        to_stake_credentials: Option<Vec<(StakeCredential, i64)>>,
        to_other_pot: Option<u64>,
    },
    RollBack {
        block_slot: u64,
        block_hash: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub context: EventContext,

    #[serde(flatten)]
    pub data: EventData,

    pub fingerprint: Option<String>,
}
