import json
from typing import Dict

polling_station_data_path = "byelections_2025/modular_polling_station_data.json"
candidates_data_path = "byelections_2025/modular_candidate_data.json"

# Load Data
with open(polling_station_data_path, "r", encoding="utf-8") as f:
    polling_stations_raw = json.load(f)

with open(candidates_data_path, "r", encoding="utf-8") as f:
    candidates = json.load(f)

def sanitize_text(val):
    """Escapes double quotes for SQL insertion."""
    if val is None:
        return ""
    return str(val).replace('"', '""').replace(chr(10), " ").strip()

def sql_int_or_null(val):
    """Returns an integer or NULL for SQL."""
    try:
        return int(val)
    except (TypeError, ValueError):
        return "NULL"

def is_valid_station(s: Dict[str, str]) -> bool:
    """Return True only if the station has valid essential fields."""
    ward = str(s.get("ward_code", "")).strip()
    code = str(s.get("polling_station_code", "")).strip()
    name = str(s.get("station_name") or s.get("polling_station_name") or "").strip()

    if not ward or not code or not name:
        return False

    rv = str(s.get("registered_voters", "0")).strip()
    return rv.isdigit()

# Filter polling stations
polling_stations = [s for s in polling_stations_raw if is_valid_station(s)]

# -------------------------------
# Deduplicate and map IDs
# -------------------------------

# Positions
positions = sorted({c["position_type"] for c in candidates if c.get("position_type")})

# Parties: unique + sequential integer IDs
unique_parties = sorted({(c["party_code"], c["party_name"], c["party_abbreviation"])
                         for c in candidates if c.get("party_code") and c.get("party_name")})
party_id_map = {code: i+1 for i, (code, _, _) in enumerate(unique_parties)}

# Counties
counties = sorted({
    (s["county_code"], s["county_name"])
    for s in polling_stations_raw
    if s.get("county_code") and s.get("county_name")
})

# Constituencies
constituencies = sorted({
    (s["constituency_code"], s["county_code"], s["constituency_name"])
    for s in polling_stations_raw
    if s.get("constituency_code")
})

# Wards
wards = sorted({
    (s["ward_code"], s["constituency_code"], s["ward_name"])
    for s in polling_stations_raw
    if s.get("ward_code") and s.get("constituency_code")
})

# Stations: sequential IDs
station_id_map = {}
station_rows = []
for i, s in enumerate(polling_stations, start=1):
    code = s.get("polling_station_code")
    if code is not None:
        sid = i
        station_id_map[str(code)] = sid
        station_rows.append((
            sid,
            int(s["ward_code"]),
            str(s["polling_station_code"]),
            sanitize_text(s.get("polling_station_name") or s.get("station_name")),
            int(s.get("registered_voters", 0))
        ))

# -------------------------------
# SQL Generation
# -------------------------------

sql_lines = []

# Positions
sql_lines.append("INSERT OR IGNORE INTO positions VALUES")
sql_lines.append(",\n".join(f'("{sanitize_text(p)}")' for p in positions) + ";")

# Parties
sql_lines.append("\nINSERT OR IGNORE INTO parties VALUES")
sql_lines.append(",\n".join(
    f'({party_id_map[code]}, "{sanitize_text(name)}", "{sanitize_text(abbr)}")'
    for code, name, abbr in unique_parties
) + ";")

# Counties
sql_lines.append("\nINSERT OR IGNORE INTO counties VALUES")
sql_lines.append(",\n".join(
    f'({int(code)}, "{sanitize_text(name)}")' for code, name in counties
) + ";")

# Constituencies
sql_lines.append("\nINSERT OR IGNORE INTO constituencies VALUES")
sql_lines.append(",\n".join(
    f'({int(code)}, {int(county_code)}, "{sanitize_text(name)}")' for code, county_code, name in constituencies
) + ";")

# -------------------------------
# Wards
# -------------------------------
sql_lines.append("\nINSERT OR IGNORE INTO wards VALUES")
sql_lines.append(",\n".join(
    f'({int(code)}, {int(const_code)}, "{sanitize_text(name)}")'
    for code, const_code, name in wards
) + ";")

# -------------------------------
# Stations
# -------------------------------
station_id_map = {}
station_rows = []
for i, s in enumerate(polling_stations, start=1):
    ward_code = s.get("ward_code")
    reg_code = s.get("polling_station_code")
    if ward_code is None or reg_code is None:
        continue  # skip invalid

    ward_code_int = int(ward_code)
    reg_code_int = int(reg_code)

    sid = i
    station_id_map[reg_code_int] = sid

    station_rows.append((
        sid,
        ward_code_int,           # FK must exist in wards
        reg_code_int,            # integer
        sanitize_text(s.get("polling_station_name") or s.get("station_name")),
        int(s.get("registered_voters", 0))
    ))

sql_lines.append("\nINSERT OR IGNORE INTO stations VALUES")
sql_lines.append(",\n".join(
    f'({sid}, {ward_code}, {reg_code}, "{name}", {registered_voters})'
    for sid, ward_code, reg_code, name, registered_voters in station_rows
) + ";")

# -------------------------------
# Candidates
# -------------------------------
sql_lines.append("\nINSERT OR IGNORE INTO candidates VALUES")
sql_lines.append(",\n".join(
    f'({i+1}, "{sanitize_text(c["name"])}", "{sanitize_text(c["gender"])}", '
    f'"{sanitize_text(c.get("photo",""))}", "{sanitize_text(c["position_type"])}", '
    f'{party_id_map.get(c["party_code"], "NULL")}, '
    f'{station_id_map.get(int(c["voting_station"]) if c.get("voting_station") else None, "NULL")})'
    for i, c in enumerate(candidates)
    if c.get("voting_station") is None or int(c["voting_station"]) in station_id_map
) + ";")

# Write SQL file
with open("../apps/cli/full_init.sql", "w", encoding="utf-8") as f:
    f.write("\n".join(sql_lines))

print("full_init.sql generated successfully.")
