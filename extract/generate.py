import json
import re
from typing import Dict, List, Tuple, Optional, Literal
import camelot
import pandas as pd
from tqdm import tqdm


CONFIG: Dict[str, Dict[str, str]] = {
    "polling_station_data": {
        "pages": "14-79"
    }
}


def extract_number_and_text(value: str) -> Tuple[Optional[str], str]:
    """
    Extract the first numeric group from a string.
    Returns:
        - number_part (str or None)
        - cleaned_text (str)
    Works for:
        - number first
        - text first
        - messy spacing
    """
    if not isinstance(value, str):
        return None, ""

    match = re.search(r"(\d+)", value)
    if not match:
        return None, value.strip()

    number = match.group(1)
    text = (value[:match.start()] + value[match.end():]).strip()
    text = re.sub(r"\s+", " ", text)

    return number, text


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
        "Const.  \nName": "constituency_name",
        "CAW \nCode": "ward_code",
        "CAW Name": "ward_name",
        "Reg. \nCentre \nCode": "reg_centre_code",
        "Registration \nCentre Name": "station_name",
        "Polling Station \nCode": "polling_station_code",
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

    # Fix cases where constituency_code is blank and county_name contains mixed text
    val = row.get("constituency_code")
    if pd.isna(val) or str(val).lower() == "nan" or val == "":
        num, text = extract_number_and_text(str(row.get("county_name", "")))
        if num:
            row["constituency_code"] = num
            row["county_name"] = text

    # Final JSON-ready cleaned format
    return {
        "county_code": str(row.get("county_code", "")).strip(),
        "county_name": str(row.get("county_name", "")).replace("\n", " ").strip(),
        "constituency_code": str(row.get("constituency_code", "")).strip(),
        "constituency_name": str(row.get("constituency_name", "")).replace("\n", " ").strip(),
        "ward_code": str(row.get("ward_code", "")).strip(),
        "ward_name": str(row.get("ward_name", "")).replace("\n", " ").strip(),
        "station_name": str(row.get("station_name", "")).replace("\n", " ").strip(),
        "polling_station_code": str(row.get("polling_station_code", "")).strip(),
        "polling_station_name": str(row.get("polling_station_name", "")).replace("\n", " ").strip(),
        "registered_voters": str(row.get("registered_voters", "")).strip(),
    }


def extract_tables_from_pdf(
    pdf_path: str,
    extraction_type: Literal["polling_station_data"],
    output_path: str,
) -> None:
    """
    Extract and clean tables from a PDF into structured JSON.

    Args:
        pdf_path: path to the PDF file.
        extraction_type: data extraction type key in CONFIG.
        output_path: where to write final JSON output.

    Raises:
        ValueError: if extraction_type is not supported or tables have unexpected structure.
    """

    if extraction_type not in CONFIG:
        raise ValueError(f"Unsupported extraction type: {extraction_type}")

    pages = CONFIG[extraction_type]["pages"]

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

    # Save JSON
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(cleaned_output, f, ensure_ascii=False, indent=4)

    print(f"Extraction complete. Saved {len(cleaned_output)} records → {output_path}")

if __name__ == "__main__":
    task_type = "polling_station_data"
    pdf_file_path = "byelections_2025.pdf"
    output_json_path = f"byelections_2025/byelections_2025_{task_type}.json"
    extract_tables_from_pdf(
        pdf_path=pdf_file_path,
        extraction_type=task_type,
        output_path=output_json_path,
    )
