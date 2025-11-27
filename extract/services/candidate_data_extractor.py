import re
import camelot
import pandas as pd
from typing import Dict,List
from services.shared_utils import extract_number_and_text

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
        df = df.rename(columns={'County Code County': 'County', '': 'County Code'})

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

    return {
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
) -> List[Dict[str, str]]:
    """
    Extract and clean tables from a PDF into structured JSON.

    Args:
        pdf_path: path to the PDF file.
        config: extraction configuration dictionary.
    Returns:
        List of cleaned candidate data dictionaries.
    """
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