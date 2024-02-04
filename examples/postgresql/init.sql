CREATE TABLE txs (
    slot INTEGER NOT NULL,
    cbor BYTEA
);

CREATE INDEX idx_txs_slot ON txs(slot);
