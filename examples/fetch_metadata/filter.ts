import * as oura from "https://raw.githubusercontent.com/txpipe/oura/v2/assets/denopkgs/v2AlphaOuraUtils/mod.ts";

export async function mapEvent(record: oura.Event) {
  if (!record.pool_registration) {
    return;
  }

  if (!record.pool_registration.pool_metadata) {
    return;
  }

  try {
    const response = await fetch(record.pool_registration.pool_metadata);
    const json = await response.json();

    return {
      tx: record.context.tx_hash,
      url: record.pool_registration.pool_metadata,
      json,
    };
  } catch (err) {
    return {
      tx: record.context.tx_hash,
      url: record.pool_registration.pool_metadata,
      err,
    };
  }
}
