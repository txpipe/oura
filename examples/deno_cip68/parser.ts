import * as oura from "./ouraTypes.ts";
import { PlutusMap, plutusMapToPlainJson } from "./plutusData.ts";

interface RefNFT {
  txHash: string;
  label: string;
  policy: string;
  metadata?: oura.GenericJson;
  version?: number;
}

function parseDatum(raw: oura.PlutusDatumRecord): Partial<RefNFT> | null {
  try {
    const [metaField, versionField, _] = raw.plutus_data.fields as [
      { map: oura.GenericJson[] },
      { int: number },
      { fields: oura.GenericJson[] }
    ];

    return {
      metadata: plutusMapToPlainJson(metaField.map as PlutusMap),
      version: versionField.int as number,
    };
  } catch (err) {
    console.error(err);
    return null;
  }
}

function extractRefNFT(
  output: oura.TxOutputRecord,
  allDatums?: oura.PlutusDatumRecord[] | null
): Partial<RefNFT> | null {
  const asset = output.assets?.find((a) => a.asset.startsWith("000"));

  if (!asset) return null;

  const datum =
    output.inline_datum ||
    allDatums?.find((d) => d.datum_hash == output.datum_hash);

  if (!datum) return null;

  return {
    label: asset.asset,
    policy: asset.policy,
    ...parseDatum(datum),
  };
}

function processTx(tx: oura.TransactionRecord) {
  return tx.outputs
    ?.map((output) => extractRefNFT(output, tx.plutus_data))
    .filter((x) => !!x)
    .map((x) => ({ ...x, txHash: tx.hash }));
}

export function mapEvent(record: oura.Event) {
  if (!record.transaction) {
    return;
  }

  return processTx(record.transaction);
}
