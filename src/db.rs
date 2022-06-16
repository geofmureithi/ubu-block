pub const SETUP: &str = r#" 
  CREATE TABLE "positions" (
    "title" TEXT PRIMARY KEY
  );
  
  CREATE TABLE "parties" (
    "id" int PRIMARY KEY,
    "title" TEXT ,
    "logo" TEXT 
  );
  
  CREATE TABLE "counties" (
    "county_code" int PRIMARY KEY,
    "county_name" TEXT 
  );
  
  CREATE TABLE "constituencies" (
    "constituency_code" int PRIMARY KEY,
    "county_code" int,
    "constituency_name" TEXT ,
    FOREIGN KEY ("county_code") REFERENCES "counties" ("county_code")
  );
  
  CREATE TABLE "wards" (
    "ward_code" int PRIMARY KEY,
    "constituency_code" int,
    "ward_name" TEXT ,
    FOREIGN KEY ("constituency_code") REFERENCES "constituencies" ("constituency_code")
  );
  
  CREATE TABLE "stations" (
    "id" int PRIMARY KEY,
    "ward_code" int,
    "reg_center_code" integer,
    "station_name" TEXT ,
    "registered_voters" integer,
    FOREIGN KEY ("ward_code") REFERENCES "wards" ("ward_code")
  );

  CREATE TABLE "candidates" (
    "id" int PRIMARY KEY,
    "title" TEXT ,
    "photo" TEXT ,
    "position_type" TEXT ,
    "party_id" int,
    "voting_station" int,
    FOREIGN KEY ("position_type") REFERENCES "positions" ("title"),
    FOREIGN KEY ("party_id") REFERENCES "parties" ("id"),
    FOREIGN KEY ("voting_station") REFERENCES "stations" ("id")
  );

  CREATE TABLE blockchain (
    height				INTEGER NOT NULL UNIQUE,
    sigkey_hash			VARCHAR NOT NULL,
    hash				VARCHAR NOT NULL PRIMARY KEY,
    hash_signature		VARCHAR NOT NULL,
    prev_hash			VARCHAR NOT NULL,
    prev_hash_signature	VARCHAR NOT NULL,
    time_accepted		INTEGER NOT NULL,
    version				INTEGER NOT NULL
  );
  CREATE INDEX blockchain_sigkey_hash ON blockchain(sigkey_hash);
  
  CREATE TABLE "results" (
    "id" int PRIMARY KEY,
    "station_id" int,
    "candidate_id" int,
    "votes" int,
    "source" TEXT ,
    FOREIGN KEY ("candidate_id") REFERENCES "candidates" ("id"),
    FOREIGN KEY ("station_id") REFERENCES "stations" ("id")
  );

  INSERT INTO positions Values ("Mca"), ("WomenRep"), ("Mp"), ("Senator"), ("Governor"), ("President");
  INSERT INTO parties Values (1, "ODM", ""), (2, "PNU", "");
  INSERT INTO counties VALUES (22, "Kiambu"), ( 45, "Kisii");
  INSERT INTO constituencies VALUES (113, 22, "Juja"), (261, 45, "Bonchari");
  INSERT INTO wards VALUES (563, 113, "Kalimoni"), (1301, 261, "Bomariba");
  INSERT INTO stations VALUES(022113056303301, 563, 33, "Athi Primary School", 533 );
  INSERT INTO stations VALUES(045261130100402, 1301, 4, "Igonga Primary School ", 685 );

  INSERT INTO candidates VALUES(1, "Mwas", "", "Mp", 1, 022113056303301), (2, "Omosh", "", "Mp", 2, 022113056303301);
  INSERT INTO candidates VALUES(3, "Adhis", "", "Mp", 1, 045261130100402), (4, "Juma", "", "Mp", 2, 045261130100402);
  INSERT INTO results VALUES(1, 022113056303301, 1, 52, "NGO1");
  INSERT INTO results VALUES(2, 022113056303301, 2, 99, "NGO1");
  INSERT INTO results VALUES(3, 045261130100402, 3, 172, "NGO2");
  INSERT INTO results VALUES(4, 022113056303301, 1, 56, "NGO1");

  INSERT INTO blockchain (hash, height, prev_hash, sigkey_hash, hash_signature, prev_hash_signature, time_accepted, version) VALUES (
    '9a0ff19183d1525a36de803047de4b73eb72506be8c81296eb463476a5c2d9e2',
    0,
    1000000000000000000000000000000000000000000000000000000000000001,
    '1:a3c07ef6cbee246f231a61ff36bbcd8e8563723e3703eb345ecdd933d7709ae2',
    '30460221008b8b3b3cfee2493ef58f2f6a1f1768b564f4c9e9a341ad42912cbbcf5c3ec82f022100fbcdfd0258fa1a5b073d18f688c2fb3d8f9a7c59204c6777f2bbf1faeb1eb1ed',
    '3046022100db037ae6cb3c6e37cbc8ec592ba7eed2e6d18e6a3caedc4e2e81581eb97acb67022100d46d8ed27b5d78a8509b1eb8549c9b6b8f1c0a134c0c7af23bb93ab8cc842e2d',
    '2022-05-06T10:38:50+02:00',
    1);


  "#;

// pub(crate) struct Database {
//     db: String,
// }

// // table positions
// enum Position {
//     Mca,
//     WomenRep,
//     Mp,
//     Senator,
//     Governor,
//     President,
// }

// // table stations
// struct PollingStation {
//     county: String,
//     county_code: String,
//     ward: String,
//     ward_code: String,
//     station_code: u64,
//     station_name: String,
//     constituency: String,
//     registered_voters: u32,
// }

// // table candidates
// struct Candidate {
//     name: String,
//     party: String,
//     role: Position,
// }

// // table results
// struct StationResult {
//     station: PollingStation,
//     candidate: Candidate,
//     votes: u64,
// }
