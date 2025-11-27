import json
import re
import camelot
import pandas as pd
from typing import Dict,List, Literal, Tuple, Optional
from services.shared_utils import extract_number_and_text

def load_and_normalize_table(df: pd.DataFrame) -> pd.DataFrame:
    """
    Normalize a Camelot-extracted DataFrame and rename columns to expected schema.
    Raises ValueError if the structure doesn't match what is expected.
    """

    if not isinstance(df, pd.DataFrame):
        raise ValueError("Expected a pandas DataFrame.")

    # First row contains column headers
    df.columns = df.iloc[0]
    df = df.drop(index=0).reset_index(drop=True)

    # Verify expected number of columns (11 for polling station layout)
    if len(df.columns) != 11:
        raise ValueError(
            f"Unexpected column count ({len(df.columns)}). "
            "Expected 11 columns for polling station extraction."
        )

    # Rename columns to stable snake_case names
    rename_map = {
        "County \nCode": "county_code",
        "County \nName": "county_name",
        "Const\n. \nCode": "constituency_code",
        "Const \nCode": "constituency_code",
        "Const.  \nName": "constituency_name",
        "CAW \nCode": "ward_code",
        "Caw \nCode": "ward_code",
        "CAW Name": "ward_name",
        "Reg. \nCentre \nCode": "reg_centre_code",
        "Registration \nCentre Name": "station_name",
        "Reg. Centre Name": "station_name",
        "Polling Station \nCode": "polling_station_code",
        "Polling \nStation Code": "polling_station_code",
        "Polling Station Name": "polling_station_name",
        "Reg. \nVoters": "registered_voters",
    }

    df = df.rename(columns=rename_map)

    return df


def clean_row(row: pd.Series) -> Dict[str, str]:
    """
    Clean and normalize a single row of extracted PDF polling-station data.
    Handles missing station names, missing codes, and text-number parsing cleanup.
    """

    row = row.copy()

    # Fix cases where polling_station_name is missing but code contains both
    val = row.get("polling_station_name")
    if pd.isna(val) or val == "":
        num, text = extract_number_and_text(str(row.get("polling_station_code", "")))
        if num:
            row["polling_station_code"] = num
            row["polling_station_name"] = text
    # Fix cases where polling_station_code is missing but name contains both
    val = row.get("polling_station_code")
    if pd.isna(val) or val == "":
        num, text = extract_number_and_text(str(row.get("polling_station_name", "")))
        if num:
            row["polling_station_code"] = num
            row["polling_station_name"] = text

        num, text = extract_number_and_text(str(row.get("station_name", "")))
        if num:
            row["polling_station_code"] = num
            row["station_name"] = text

    # Fix cases where constituency_code is blank and county_name contains mixed text
    val = row.get("constituency_code")
    if pd.isna(val) or str(val).lower() == "nan" or val == "":
        num, text = extract_number_and_text(str(row.get("county_name", "")))
        if num:
            row["constituency_code"] = num
            row["county_name"] = text

    #Fix cases where the constituency_name is blank and constituency_code contains both text and number
    val = row.get("constituency_name")
    if pd.isna(val) or str(val).lower() == "nan" or val == "":
        num, text = extract_number_and_text(str(row.get("ward_code", "")))
        if num:
            row["ward_code"] = num
            row["constituency_name"] = text
    
    #Fix cases where the registered_voters column is empty but the polling_station_name column contains both ward name and registered voters
    val = row.get("registered_voters")
    if pd.isna(val) or str(val).lower() == "nan" or val == "":
        num, text = extract_number_and_text(str(row.get("polling_station_name", "")))
        if num:
            row["registered_voters"] = num
            row["polling_station_name"] = text

    #Fix cases where the ward_code is blank and the constituency_name contains both text and number
    val = row.get("ward_code")
    if pd.isna(val) or str(val).lower() == "nan" or val == "":
        num, text = extract_number_and_text(str(row.get("constituency_name", "")))
        if num:
            row["ward_code"] = num
            row["constituency_name"] = text

        number, text = extract_number_and_text(str(row.get("ward_name", "")))
        if number:
            row["ward_code"] = number
            row["ward_name"] = text

    #fix cases where the ward_name is blank and the ward_code contains both text and number
    val = row.get("ward_name")
    if pd.isna(val) or str(val).lower() == "nan" or val == "":
        num, text = extract_number_and_text(str(row.get("ward_code", "")))
        if num:
            row["ward_code"] = num
            row["ward_name"] = text

    # Final JSON-ready cleaned format
    return {
        "county_code": str(row.get("county_code", "")).strip(),
        "county_name": str(row.get("county_name", "")).replace("\n", " ").strip(),
        "constituency_code": str(row.get("constituency_code", "")).strip(),
        "constituency_name": str(row.get("constituency_name", "")).replace("\n", " ").strip(),
        "ward_code": str(row.get("ward_code", "")).strip(),
        "ward_name": str(row.get("ward_name", "")).replace("\n", " ").strip(),
        "station_name": str(row.get("station_name", "")).replace("\n", " ").strip(),
        "polling_station_code": str(row.get("polling_station_code", "")).replace("\n", "").strip(),
        "polling_station_name": str(row.get("polling_station_name", "")).replace("\n", " ").strip(),
        "registered_voters": str(row.get("registered_voters", "")).strip(),
    }
def extract_polling_data_tables_from_pdf(
    pdf_path: str,
    config: Dict[str, str],
) -> List[Dict[str, str]]:
    """
    Extract and clean tables from a PDF into structured JSON.

    Args:
        pdf_path: path to the PDF file.
        config: extraction configuration dictionary.
    Returns:
        List of cleaned polling station data dictionaries.

    """

    pages = config["pages"]
    print(f"Extracting tables from PDF (pages {pages})…")
    tables = camelot.read_pdf(pdf_path, pages=pages)

    normalized_frames: List[pd.DataFrame] = []

    for table in tables:
        df = load_and_normalize_table(table.df)
        normalized_frames.append(df)

    # Combine all extracted tables
    combined = pd.concat(normalized_frames, ignore_index=True)

    # Clean rows one by one
    cleaned_output = [clean_row(row) for _, row in combined.iterrows()]

    return cleaned_output