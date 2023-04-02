import * as oura from "https://raw.githubusercontent.com/txpipe/oura/v2/assets/denopkgs/v2AlphaOuraUtils/mod.ts";

async function fetchOffchain(url?: string | null) {
  if (!url) {
    return null;
  }

  try {
    const response = await fetch(url);
    const body = await response.json();

    return {
      status: response.status,
      url,
      body,
    };
  } catch (err) {
    return {
      url,
      error: err.toString(),
    };
  }
}

export async function mapEvent(record: oura.Event) {
  if (!record.pool_registration) {
    return;
  }

  const offchain = await fetchOffchain(record.pool_registration.pool_metadata);

  return {
    tx: record.context.tx_hash,
    onchain: record.pool_registration,
    offchain,
  };
}
