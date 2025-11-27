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
    name = str(s.get("station_name", "") or s.get("polling_station_name", "")).strip()

    if not ward or not code or not name:
        return False

    rv = str(s.get("registered_voters", "0")).strip()
    if not rv.isdigit():
        return False

    return True

# Filter polling stations
polling_stations = [s for s in polling_stations_raw if is_valid_station(s)]

# Collect unique values
positions = sorted({p for p in (c["position_type"] for c in candidates) if p})

parties = sorted({
    (c["party_code"], c["party_abbreviation"], c["party_abbreviation"])
    for c in candidates
    if c.get("party_code") and c.get("party_name")
})

counties = sorted({
    (c["county_code"], c["county_name"])
    for c in candidates
    if c.get("county_code") and c.get("county_name")
})

constituencies = sorted({
    (c["constituency_code"], c["county_code"], c["constituency_name"])
    for c in candidates
    if c.get("constituency_code") and c.get("county_code") and c.get("constituency_name")
})

wards = sorted({
    (c["ward_code"], c["constituency_code"], c["ward_name"])
    for c in candidates
    if c.get("ward_code") and c.get("constituency_code") and c.get("ward_name")
})

# ---------------------------------------------------------
# SQL Generation
# ---------------------------------------------------------
sql_lines = []

# Positions
sql_lines.append("INSERT INTO positions VALUES")
sql_lines.append(",\n".join(f'("{sanitize_text(p)}")' for p in positions) + ";")

# Parties
sql_lines.append("\nINSERT INTO parties VALUES")
sql_lines.append(",\n".join(
    f'({code}, "{sanitize_text(name)}", "{sanitize_text(abbr)}")'
    for code, name, abbr in parties
) + ";")

# Counties
sql_lines.append("\nINSERT INTO counties VALUES")
sql_lines.append(",\n".join(
    f'({code}, "{sanitize_text(name)}")'
    for code, name in counties
) + ";")

# Constituencies
sql_lines.append("\nINSERT INTO constituencies VALUES")
sql_lines.append(",\n".join(
    f'({code}, {county_code}, "{sanitize_text(name)}")'
    for code, county_code, name in constituencies
) + ";")

# Wards
sql_lines.append("\nINSERT INTO wards VALUES")
sql_lines.append(",\n".join(
    f'({code}, {const_code}, "{sanitize_text(name)}")'
    for code, const_code, name in wards
) + ";")

# Generate sequential IDs for stations (skip any None codes)
station_id_map = {}
for i, s in enumerate(polling_stations, start=1):
    code = s.get("polling_station_code")
    if code is not None:
        station_id_map[int(code)] = i

# SQL for stations using sequential ID
sql_lines.append("\nINSERT INTO stations VALUES")
sql_lines.append(",\n".join(
    f'({station_id_map[int(s["polling_station_code"])]}, '
    f'{int(s["ward_code"])}, '
    f'{int(s["polling_station_code"])}, '
    f'"{sanitize_text(s.get("polling_station_name") or s.get("station_name"))}", '
    f'{int(s.get("registered_voters", 0))})'
    for s in polling_stations
) + ";")

# Candidates referencing the mapped station ID
sql_lines.append("\nINSERT INTO candidates VALUES")
sql_lines.append(",\n".join(
    f'({i+1}, '
    f'"{sanitize_text(c["name"])}", '
    f'"{sanitize_text(c["gender"])}", '
    f'"{sanitize_text(c.get("photo", ""))}", '
    f'"{sanitize_text(c["position_type"])}", '
    f'{c["party_code"]}, '
    f'{station_id_map.get(int(c.get("voting_station")) if c.get("voting_station") is not None else None, "NULL")})'
    for i, c in enumerate(candidates)
) + ";")


# Write to file
with open("init.sql", "w", encoding="utf-8") as f:
    f.write("\n".join(sql_lines))

print("init.sql generated successfully.")
