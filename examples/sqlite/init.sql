CREATE TABLE txs (
    slot INTEGER NOT NULL,
    cbor BLOB
);

CREATE INDEX idx_txs_slot ON txs(slot);
