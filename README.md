# ubu-block

Uchaguzi bila Ubaguzi Blockchain

<img src="./assets/submit2.png">

A transparent, tamper-evident, and community-driven blockchain platform for election results.
It is currently focused on Kenyan election's data but can easily be adapted for other countries.

> [!NOTE]
> This code is actively in active development and may break. PRs welcome

## Node Types

- **Submission Nodes:** Allow users to submit polling station results.
- **Observer Nodes:** Provide public, read-only access for transparency and independent verification.
- **Verification Nodes:** Cross-reference submitted results with official sources and participate in reward submission.

## Features

1. **Immutable:** You can't make changes, only append—no `fungua server` and other shenanigans.
2. **Distributed:** No single point of data storage.
3. **Accessible:** Runs on SQLite, widely supported.
4. **Community Driven:** Dynamic number of signatories.
5. **Node Roles:** Submission, Observer, and Verification nodes for robust, transparent operations.
6. **BFT Consensus:** Tolerates up to 1/3 malicious nodes.
7. **Audit Trail:** All submissions (accepted or rejected) are recorded.
8. **Encrypted Communication:** End-to-end encryption for all node interactions.

## Getting Started

```sh
git clone <repo> ubu-block
cd ubu-block
mkdir data
```

If you want to run binaries without installing Rust:

1. Get the latest binary from the releases page.
2. Replace `cargo run` with `ubu-block`
3. Download the relevant files (see "Setup initial files").

### Setup initial files

```sh
mkdir data

# Create the db files
cp src/sql/empty.db data/blockchain.db
cp src/sql/empty.db data/private.db
```

### Initialize a Blockchain

```sh
cargo run init --init setup_constituencies.sql
```

You should get:

```
INFO  ubu_block] Blockchain was successfully initialized!
```

### Add Some Blocks

Currently `ubu-block` has very limited dummy data, but this should change as soon as IEBC publishes the voter register.

For testing purposes, limited data from previous by-elections is used. See `src/sql/main_db.sql`.

```sh
cargo run insert --station 022113056303301 --candidate 1 --votes 66
cargo run insert --station 022113056303301 --candidate 2 --votes 21
```

You should get:

```
INFO ubu_block] Block was added successfully!
```

### Validate the Blockchain

```sh
cargo run validate
```

You should get:

```
INFO ubu_block] Blockchain is valid!
```

### Querying

```sh
cargo run query -q "SELECT
  c.name as candidate,
  SUM(votes) as votes,
  ward_name as ward,
  constituency_name as constituency,
  county_name as county,
  parties.title as party
FROM
  results
  INNER JOIN stations ON stations.id = results.station_id
  INNER JOIN candidates c ON c.id = results.candidate_id
  INNER JOIN wards on stations.ward_code = wards.ward_code
  INNER JOIN parties ON parties.id = c.party_id
  INNER JOIN constituencies ON wards.constituency_code = constituencies.constituency_code
  INNER JOIN counties ON constituencies.county_code = counties.county_code
WHERE
  position_type = 'Mp'  and constituency = 'Juja' GROUP BY candidate ;"
```

**_NOTE:_** Each query must return the following columns: candidate, votes, ward, constituency, county, party. This is a temporary limitation.

You should get:

```
+--------+--------------+----------+-----------+-------+-------+
| county | constituency |   ward   | candidate | party | votes |
+--------+--------------+----------+-----------+-------+-------+
| Kiambu |     Juja     | Kalimoni |   Omosh   |  ODM  |  21   |
| Kiambu |     Juja     | Kalimoni |   Mwas    |  PNU  |  66   |
+--------+--------------+----------+-----------+-------+-------+
```

### Attempting to Tamper with Results

Since immutability is a core goal, let's try to edit votes for `Omosh` and see if we can get away with it.

Open `blockchain.db` with your favorite SQLite editor and run:

```sql
UPDATE "results" SET "votes"= 71 WHERE _rowid_ =1
```

Running `query` again:

```
+--------+--------------+----------+-----------+-------+-------+
| county | constituency |   ward   | candidate | party | votes |
+--------+--------------+----------+-----------+-------+-------+
| Kiambu |     Juja     | Kalimoni |   Omosh   |  ODM  |  71   |
| Kiambu |     Juja     | Kalimoni |   Mwas    |  PNU  |  66   |
+--------+--------------+----------+-----------+-------+-------+
```
Let's try validating our blockchain:

```
cargo run validate

thread 'main' panicked at 'Could not verify block, found 0e70cebe0ab3bd8c3606a08d26483d092534eea4ccdb7816fc2692aee5ed3109, block: Block {... CandidateResult { station_id: 22113056303301, candidate_id: 1, votes: 71 }]......', src/db.rs:189:17
```

How about that? No `fungua servers` and everything is public and SQL-friendly.

## Free Public Servers

Below are the servers you are using for free. This may change over time. If you are not close to one of these, your network may be slow.

| Location | Vendor | Specification | IP |
| -------- | ------ | ------------- | --- |
| Frankfurt   | Digital Ocean  | 2 VCPU / 2GB RAM | 157.***.***.***


## Roadmap

### v0.5
- [ ] Verification node: Reward distribution for correct submissions

### v0.4

- [ ] Verification node: Cross-reference submitted results with official sources
- [ ] Encrypted node-to-node communication

### v0.3

- [x] Submission node: Allow authorized users to submit station results
- [x] HTTP API
- [x] Mobile and Web apps

### v0.2

- [x] P2P - ability to add nodes
- [x] Observer node: Public read-only access for transparency
- [x] Setup initial regional data generically
- [x] Merkle tree implementation
- [ ] Basic integration testing
- [x] Tabling of SQL results

### v0.1

- [x] Clap
- [x] Database, SQLite
- [x] Blockchain
- [x] CI/CD


## References

- [Do you need a blockchain?](https://eprint.iacr.org/2017/375.pdf)
- [Daisy](https://github.com/ivoras/daisy)

## Credits

- [Free Stock Images from Pexels](https://www.pexels.com/)
