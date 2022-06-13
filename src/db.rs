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
