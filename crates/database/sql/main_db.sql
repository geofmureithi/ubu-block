CREATE TABLE "positions" ("title" TEXT PRIMARY KEY);

CREATE TABLE "parties" (
    "id" int PRIMARY KEY,
    "title" TEXT,
    "logo" TEXT
);

CREATE INDEX parties_title ON parties(title);

CREATE TABLE "counties" (
    "county_code" int PRIMARY KEY,
    "county_name" TEXT
);

CREATE INDEX counties_name ON counties(county_name);

CREATE TABLE "constituencies" (
    "constituency_code" int PRIMARY KEY,
    "county_code" int,
    "constituency_name" TEXT,
    FOREIGN KEY ("county_code") REFERENCES "counties" ("county_code")
);

CREATE INDEX constituencies_county_code ON constituencies(county_code);

CREATE INDEX constituencies_name ON constituencies(constituency_name);

CREATE TABLE "wards" (
    "ward_code" int PRIMARY KEY,
    "constituency_code" int,
    "ward_name" TEXT,
    FOREIGN KEY ("constituency_code") REFERENCES "constituencies" ("constituency_code")
);

CREATE INDEX wards_constituency_code ON wards(constituency_code);

CREATE INDEX wards_name ON wards(ward_name);

CREATE TABLE "stations" (
    "id" int PRIMARY KEY,
    "ward_code" int,
    "reg_center_code" integer,
    "station_name" TEXT,
    "registered_voters" integer,
    FOREIGN KEY ("ward_code") REFERENCES "wards" ("ward_code")
);

CREATE INDEX stations_ward_code ON stations(ward_code);

CREATE INDEX stations_station_name ON stations(station_name);

CREATE INDEX stations_reg_center_code ON stations(reg_center_code);

CREATE TABLE "candidates" (
    "id" int PRIMARY KEY,
    "name" TEXT NOT NULL,
    "gender" TEXT NOT NULL,
    "photo" TEXT,
    "position_type" TEXT NOT NULL,
    "party_id" int NOT NULL,
    "voting_station" int,
    FOREIGN KEY ("position_type") REFERENCES "positions" ("title"),
    FOREIGN KEY ("party_id") REFERENCES "parties" ("id"),
    FOREIGN KEY ("voting_station") REFERENCES "stations" ("id")
);

CREATE INDEX candidates_name ON candidates("name");

CREATE INDEX candidates_gender ON candidates(gender);

CREATE INDEX candidates_position_type ON candidates(position_type);

CREATE INDEX candidates_party_id ON candidates(party_id);

CREATE INDEX candidates_voting_station ON candidates(voting_station);

CREATE TABLE pubkeys (
    pubkey_hash VARCHAR NOT NULL PRIMARY KEY,
    creator VARCHAR NOT NULL UNIQUE,
    pubkey VARCHAR NOT NULL,
    state CHAR NOT NULL,
    time_added INTEGER NOT NULL,
    time_revoked INTEGER,
    block_height INTEGER NOT NULL,
    metadata VARCHAR -- JSON
);

CREATE TABLE blockchain (
    height INTEGER NOT NULL UNIQUE,
    sigkey_hash VARCHAR NOT NULL,
    timestamp INTEGER NOT NULL,
    hash VARCHAR NOT NULL PRIMARY KEY,
    hash_signature VARCHAR NOT NULL,
    prev_hash VARCHAR NOT NULL,
    prev_hash_signature VARCHAR NOT NULL,
    version INTEGER NOT NULL,
    merkle_root BLOB NOT NULL,
    FOREIGN KEY("sigkey_hash") REFERENCES "pubkeys" ("pubkey_hash")
);

CREATE INDEX blockchain_sigkey_hash ON blockchain(sigkey_hash);

CREATE TABLE peers (
    address VARCHAR NOT NULL PRIMARY KEY,
    -- in the format "address:port", lowercase
    time_added INTEGER NOT NULL,
    -- time last seen
    permanent BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE "results" (
    "station_id" int NOT NULL,
    "candidate_id" int NOT NULL,
    "votes" int NOT NULL,
    "block_height" int NOT NULL,
    FOREIGN KEY ("candidate_id") REFERENCES "candidates" ("id"),
    FOREIGN KEY ("station_id") REFERENCES "stations" ("id"),
    FOREIGN KEY ("block_height") REFERENCES "blockchain" ("height")
);

-- Prevent exceeding registered voters in a station
CREATE TRIGGER limit_votes_per_station
BEFORE INSERT ON results
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN (NEW.votes + IFNULL((SELECT SUM(votes) 
                                      FROM results 
                                      WHERE station_id = NEW.station_id), 0))
                 > (SELECT registered_voters 
                    FROM stations 
                    WHERE id = NEW.station_id)
            THEN RAISE(ABORT, 'Vote count exceeds registered voters for this station')
        END;
END;

-- Optional: also prevent updates that would exceed the limit
CREATE TRIGGER limit_votes_update
BEFORE UPDATE OF votes ON results
FOR EACH ROW
BEGIN
    SELECT
        CASE
            WHEN (NEW.votes + IFNULL((SELECT SUM(votes) 
                                      FROM results 
                                      WHERE station_id = NEW.station_id AND id != OLD.id), 0))
                 > (SELECT registered_voters 
                    FROM stations 
                    WHERE id = NEW.station_id)
            THEN RAISE(ABORT, 'Vote count exceeds registered voters for this station')
        END;
END;

CREATE INDEX results_station_id ON results(station_id);

CREATE INDEX results_candidate_id ON results(candidate_id);

CREATE INDEX results_votes ON results(votes);

CREATE INDEX results_block_height ON results(block_height);
