CREATE TABLE privkeys (
    pubkey_hash VARCHAR NOT NULL PRIMARY KEY,
    privkey VARCHAR NOT NULL,
    time_added INTEGER NOT NULL
);