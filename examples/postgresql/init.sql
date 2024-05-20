-- Table for storing CBOR blocks
CREATE TABLE blocks (
    slot INTEGER NOT NULL,
    cbor BYTEA
);

-- Index for the blocks table
CREATE INDEX idx_blocks_slot ON blocks(slot);

-- Table for storing CBOR transactions
CREATE TABLE txs (
    slot INTEGER NOT NULL,
    cbor BYTEA
);

-- Index for the txs table
CREATE INDEX idx_txs_slot ON txs(slot);