export type Era =
  | "Undefined"
  | "Unknown"
  | "Byron"
  | "Shelley"
  | "Allegra"
  | "Mary"
  | "Alonzo"
  | "Babbage";

export type GenericJson = Record<string, unknown>;

export type MetadatumRendition = {
  map_json?: GenericJson;
  array_json?: GenericJson;
  int_scalar?: string;
  text_scalar?: string;
  bytes_hex?: string;
};

export type MetadataRecord = {
  label: string;
  content: MetadatumRendition;
};

export type CIP25AssetRecord = {
  version: string;
  policy: string;
  asset: string;
  name: string | null;
  image: string | null;
  media_type: string | null;
  description: string | null;
  raw_json: GenericJson;
};

export type CIP15AssetRecord = {
  voting_key: string;
  stake_pub: string;
  reward_address: string;
  nonce: number;
  raw_json: GenericJson;
};

export type TxInputRecord = {
  tx_id: string;
  index: number;
};

export type OutputAssetRecord = {
  policy: string;
  asset: string;
  asset_ascii: string | null;
  amount: number;
};

export type TxOutputRecord = {
  address: string;
  amount: number;
  assets: OutputAssetRecord[] | null;
  datum_hash: string | null;
  inline_datum: PlutusDatumRecord | null;
};

export type MintRecord = {
  policy: string;
  asset: string;
  quantity: number;
};

export type WithdrawalRecord = {
  reward_account: string;
  coin: number;
};

export type TransactionRecord = {
  hash: string;
  fee: number;
  ttl: number | null;
  validity_interval_start: number | null;
  network_id: number | null;
  input_count: number;
  collateral_input_count: number;
  has_collateral_output: boolean;
  output_count: number;
  mint_count: number;
  total_output: number;

  // include_details
  metadata: MetadataRecord[] | null;
  inputs: TxInputRecord[] | null;
  outputs: TxOutputRecord[] | null;
  collateral_inputs: TxInputRecord[] | null;
  collateral_output: TxOutputRecord | null;
  mint: MintRecord[] | null;
  vkey_witnesses: VKeyWitnessRecord[] | null;
  native_witnesses: NativeWitnessRecord[] | null;
  plutus_witnesses: PlutusWitnessRecord[] | null;
  plutus_redeemers: PlutusRedeemerRecord[] | null;
  plutus_data: PlutusDatumRecord[] | null;
  withdrawals: WithdrawalRecord[] | null;
  size: number;
};

export type EventContext = {
  block_hash: string | null;
  block_number: number | null;
  slot: number | null;
  timestamp: number | null;
  tx_idx: number | null;
  tx_hash: string | null;
  input_idx: number | null;
  output_idx: number | null;
  output_address: string | null;
  certificate_idx: number | null;
};

export type StakeCredential = {
  addr_keyhash?: string;
  scripthash?: string;
};

export type VKeyWitnessRecord = {
  vkey_hex: string;
  signature_hex: string;
};

export type NativeWitnessRecord = {
  policy_id: string;
  script_json: GenericJson;
};

export type PlutusWitnessRecord = {
  script_hash: string;
  script_hex: string;
};

export type PlutusRedeemerRecord = {
  purpose: string;
  ex_units_mem: number;
  ex_units_steps: number;
  input_idx: number;
  plutus_data: GenericJson;
};

export type PlutusDatumRecord = {
  datum_hash: string;
  plutus_data: GenericJson;
};

export type BlockRecord = {
  era: Era;
  epoch: number | null;
  epoch_slot: number | null;
  body_size: number;
  issuer_vkey: string;
  vrf_vkey: string;
  tx_count: number;
  slot: number;
  hash: string;
  number: number;
  previous_hash: string;
  cbor_hex: string | null;
  transactions: TransactionRecord[] | null;
};

export type CollateralRecord = {
  tx_id: string;
  index: number;
};

export type PoolRegistrationRecord = {
  operator: string;
  vrf_keyhash: string;
  pledge: number;
  cost: number;
  margin: number;
  reward_account: string;
  pool_owners: string[];
  relays: string[];
  pool_metadata: string | null;
  pool_metadata_hash: string | null;
};

export type RollBackRecord = {
  block_slot: number;
  block_hash: string;
};

export type MoveInstantaneousRewardsCertRecord = {
  from_reserves: boolean;
  from_treasury: boolean;
  to_stake_credentials: Array<[StakeCredential, number]> | null;
  to_other_pot: number | null;
};

export type NativeScriptRecord = {
  policy_id: string;
  script: GenericJson;
};

export type PlutusScriptRecord = {
  hash: string;
  data: string;
};

export type StakeRegistrationRecord = { credential: StakeCredential };

export type StakeDeregistrationRecord = { credential: StakeCredential };

export type StakeDelegation = {
  credential: StakeCredential;
  pool_hash: string;
};

export type PoolRetirementRecord = {
  pool: string;
  epoch: number;
};

export type GenesisKeyDelegationRecord = {};

export type Event = {
  context: EventContext;
  fingerprint?: string;

  block?: BlockRecord;
  block_end?: BlockRecord;
  transaction?: TransactionRecord;
  transaction_end?: TransactionRecord;
  tx_input?: TxInputRecord;
  tx_output?: TxOutputRecord;
  output_asset?: OutputAssetRecord;
  metadata?: MetadataRecord;
  v_key_witness?: VKeyWitnessRecord;
  native_witness?: NativeWitnessRecord;
  plutus_witness?: PlutusWitnessRecord;
  plutus_redeemer?: PlutusRedeemerRecord;
  plutus_datum?: PlutusDatumRecord;
  cip25_asset?: CIP25AssetRecord;
  cip15_asset?: CIP15AssetRecord;
  mint?: MintRecord;
  collateral?: CollateralRecord;
  native_script?: NativeScriptRecord;
  plutus_script?: PlutusScriptRecord;
  stake_registration?: StakeRegistrationRecord;
  stake_deregistration?: StakeDeregistrationRecord;
  stake_delegation?: StakeDelegation;
  pool_registration?: PoolRegistrationRecord;
  pool_retirement?: PoolRetirementRecord;
  genesis_key_delegation?: GenesisKeyDelegationRecord;
  move_instantaneous_rewards_cert?: MoveInstantaneousRewardsCertRecord;
  roll_back?: RollBackRecord;
};
