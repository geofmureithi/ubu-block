import re
import camelot
import pandas as pd
from typing import Dict,List
from services.shared_utils import extract_number_and_text

# Global polling station index (loaded once)
POLLING_INDEX = {
    "by_ward": {},
    "by_constituency": {},
    "by_county": {},
}

def load_polling_index(polling_stations: List[Dict]):
    """
    Build fast-lookup indices to pick canonical polling stations.
    """
    from collections import defaultdict

    by_ward = defaultdict(list)
    by_constituency = defaultdict(list)
    by_county = defaultdict(list)

    for s in polling_stations:
        by_ward[s.get("ward_code")].append(s)
        by_constituency[s.get("constituency_code")].append(s)
        by_county[s.get("county_code")].append(s)

    POLLING_INDEX["by_ward"] = by_ward
    POLLING_INDEX["by_constituency"] = by_constituency
    POLLING_INDEX["by_county"] = by_county

def pick_canonical_station(candidate):
    """
    Select a representative polling station based on candidate level.
    Primary lookup by code; fallback by normalized name if code fails.
    Returns a single station dict or None.
    """
    pos = candidate["position_type"]

    ward_code = candidate.get("ward_code")
    const_code = candidate.get("constituency_code")
    county_code = candidate.get("county_code")

    ward_name = candidate.get("ward_name", "")
    const_name = candidate.get("constituency_name", "")
    county_name = candidate.get("county_name", "")

    # Helper to normalize names
    def normalize_name(name: str) -> str:
        return str(name).lower().replace(" constituency", "").replace(".", "").strip()

    stations = []

    # MCA → ward-level lookup
    if pos == "Member of County Assembly" and ward_code:
        stations = POLLING_INDEX["by_ward"].get(ward_code, [])
        if not stations and ward_name:
            # fallback by name
            for s_list in POLLING_INDEX["by_ward"].values():
                for station in s_list:
                    if normalize_name(station["ward_name"]) == normalize_name(ward_name):
                        stations.append(station)

    # MP → constituency-level lookup
    elif pos == "Member of Parliament" and const_code:
        stations = POLLING_INDEX["by_constituency"].get(const_code, [])
        if not stations and const_name:
            # fallback by name
            for s_list in POLLING_INDEX["by_constituency"].values():
                for station in s_list:
                    if normalize_name(station["constituency_name"]) == normalize_name(const_name):
                        stations.append(station)

    # Senator / Governor / Women Rep → county-level lookup
    elif pos in ["Senator", "Governor", "Women Rep"] and county_code:
        stations = POLLING_INDEX["by_county"].get(county_code, [])
        if not stations and county_name:
            # fallback by name
            for s_list in POLLING_INDEX["by_county"].values():
                for station in s_list:
                    if normalize_name(station["county_name"]) == normalize_name(county_name):
                        stations.append(station)

    # ultimate fallback → pick any county-level station
    else:
        stations = POLLING_INDEX["by_county"].get(county_code, [])

    # return first station dict or None
    return stations[0] if stations else None


def fix_inline_spaces(text: str) -> str:
    if not isinstance(text, str):
        return text

    # 1. Collapse multiple spaces into single space
    text = re.sub(r'\s+', ' ', text)

    # 2. Fix cases where a single letter gets split from a word (KENY A → KENYA)
    text = re.sub(r'(\w)\s+(\w)$', r'\1\2', text)

    # 3. Remove stray spaces around hyphens (FORD- KENY A → FORD-KENYA)
    text = re.sub(r'\s*-\s*', '-', text)

    return text.strip()

def safe_get(row, keys, default=""):
    """
    Safely return the first available column from a list of possible keys.
    Example:
        safe_get(row, ['County \nNames', 'County'])
    """
    for key in keys:
        if key in row and pd.notna(row[key]):
            return str(row[key]).replace('\n', ' ').strip()
    return default


def load_and_normalize_table(df: pd.DataFrame) -> pd.DataFrame:
    """
    Normalize a Camelot-extracted DataFrame and rename columns to expected schema.
    """

    if not isinstance(df, pd.DataFrame):
        raise ValueError("Expected a pandas DataFrame.")

    # First row contains column headers
    df.columns = df.iloc[0]
    df = df.drop(index=0).reset_index(drop=True)
    normalized_cols = [c.replace("\n", " ").replace("  ", " ").strip() for c in df.columns]
    df.columns = normalized_cols

    # fix the country code county issue
    if 'County Code County' in df.columns:
        df = df.rename(columns={'County Code County': 'County Code', '': 'County'})

    # Rename columns to stable snake_case names
    rename_map = {
        "Other Names": "Other",
        "County Code": "county_code",
        "County Names": "county_name",
        "County": "county_name",
        "County Name": "county_name",
        "Const Code": "constituency_code",
        "Constituency Code": "constituency_code",
        "Const. Code": "constituency_code",
        "Const.  Name": "constituency_name",
        "Constituency Name": "constituency_name",
        "CAW Code": "ward_code",
        "CAW Name": "ward_name",
        "CAW": "ward_name",
        "Party Abrv": "party_abbreviation",
        "Party Abrv.": "party_abbreviation",
        "Party Abrr.": "party_abbreviation",
        "Party Name": "party_name",
        "Party Code": "party_code",
    }

    df = df.rename(columns=rename_map)
    return df

def build_candidate_record(row, position_type):
    """
    Build a clean candidate dictionary from a row.
    Handles messy column names, missing values, and table differences.
    """
    surname = safe_get(row, ["Surname"])
    other_names = safe_get(row, ["Other"])
    
    full_name = " ".join(part for part in [other_names, surname] if part).strip()

    party_code = safe_get(row, ["party_code"])
    party_abbreviation = safe_get(row, ["party_abbreviation"])
    party_name = safe_get(row, ["party_name"])

    photo = safe_get(row, ["Symbol"], default=None)
    if photo == "":
        photo = None

    county_code = safe_get(row, ["county_code"])
    county_name = safe_get(row, ["county_name"])


    constituency_code = safe_get(row, ["constituency_code"])
    constituency_name = safe_get(row, ["constituency_name"])

    ward_code = safe_get(row, ["ward_code"])
    ward_name = safe_get(row, ["ward_name"])

    if pd.isna(ward_name) or str(ward_name).lower() == "nan" or ward_name == "":
        num, text = extract_number_and_text(str(row.get("ward_code", "")))
        if num:
            ward_code = num
            ward_name = text

    if pd.isna(ward_code) or str(ward_code).lower() == "nan" or ward_code == "":
        num, text = extract_number_and_text(str(row.get("ward_name", "")))
        if num:
            ward_code = num
            ward_name = text

    record =  {
        "name": full_name,
        "gender": "unknown",              # not provided in PDF
        "photo": photo,                   # always empty in PDF
        "position_type": position_type,   # inferred based on table structure

        "party_name": fix_inline_spaces(party_name),
        "party_abbreviation": fix_inline_spaces(party_abbreviation),
        "party_code": fix_inline_spaces(party_code),

        "county_code": county_code,
        "county_name": fix_inline_spaces(county_name),

        "constituency_code": constituency_code,
        "constituency_name": fix_inline_spaces(constituency_name),

        "ward_code": ward_code,
        "ward_name": fix_inline_spaces(ward_name),
    }

    station = pick_canonical_station(record)
    record["voting_station"] = (
        station.get('polling_station_code') if station else None
    )
    return record
def get_position_type_from_columns(df: pd.DataFrame) -> str:
    """
    Infer the candidate position type based on the presence of certain columns.
    """
    # Infer position type based on presence of certain columns
    if any("ward" in col for col in df.columns):
        position_type = "Member of County Assembly"

    elif any("constituency" in col for col in df.columns):
        position_type = "Member of Parliament"

    elif any("county" in col for col in df.columns):
        position_type = "Senator"

    else:
        position_type = "Unknown"
    return position_type
def extract_candidate_data_tables_from_pdf(
    pdf_path: str,
    config: Dict[str, str],
    polling_stations: List[Dict[str, str]],
) -> List[Dict[str, str]]:
    """
    Extract and clean tables from a PDF into structured JSON.

    Args:
        pdf_path: path to the PDF file.
        config: extraction configuration dictionary.
    Returns:
        List of cleaned candidate data dictionaries.
    """
    load_polling_index(polling_stations)
    pages = config["pages"]
    print(f"Extracting tables from PDF (pages {pages})…")
    tables = camelot.read_pdf(pdf_path, pages=pages)
    cleaned_output: List[Dict[str, str]] = []
    for table in tables:
        df = load_and_normalize_table(table.df)
        print(df.columns)
        position = get_position_type_from_columns(df)
        candidate_record = [build_candidate_record(row, position) for _, row in df.iterrows()]
        cleaned_output.extend(candidate_record)

    return cleaned_output 