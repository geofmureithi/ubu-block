INSERT INTO
    positions
Values
    ("Mca"),
    ("WomenRep"),
    ("Mp"),
    ("Senator"),
    ("Governor"),
    ("President");

INSERT INTO
    parties
Values
    (1, "ODM", ""),
    (2, "PNU", "");

INSERT INTO
    counties
VALUES
    (22, "Kiambu"),
    (45, "Kisii");

INSERT INTO
    constituencies
VALUES
    (113, 22, "Juja"),
    (261, 45, "Bonchari");

INSERT INTO
    wards
VALUES
    (563, 113, "Kalimoni"),
    (1301, 261, "Bomariba");

INSERT INTO
    stations
VALUES
    (
        022113056303301,
        563,
        33,
        "Athi Primary School",
        533
    );

INSERT INTO
    stations
VALUES
    (
        045261130100402,
        1301,
        4,
        "Igonga Primary School",
        685
    );

INSERT INTO
    candidates
VALUES
    (1, "Mwas", "M", "", "Mp", 1, 022113056303301),
    (2, "Omosh", "M", "", "Mp", 2, 022113056303301);

INSERT INTO
    candidates
VALUES
    (3, "Adhis", "F", "", "Mp", 1, 045261130100402),
    (4, "Juma", "F", "", "Mp", 2, 045261130100402);
