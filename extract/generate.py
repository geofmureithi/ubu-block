import json
import argparse
from typing import Dict, Literal, Optional
from services.polling_data_extractor import extract_polling_data_tables_from_pdf
from services.candidate_data_extractor import extract_candidate_data_tables_from_pdf

CONFIG: Dict[str, Dict[str, str]] = {
    "polling_station_data": {
        "pages": "14-79"
    },
    "candidate_data": {
        "pages": "1-13"
    },
}

def extract_tables_from_pdf(
    pdf_path: str,
    extraction_type: Literal["polling_station_data", "candidate_data"],
    output_path: str,
    polling_stations_json: Optional[str] = None,
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

    if extraction_type == "polling_station_data":
        cleaned_output = extract_polling_data_tables_from_pdf(
            pdf_path=pdf_path,
            config=CONFIG[extraction_type],
        )
        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(cleaned_output, f, ensure_ascii=False, indent=4)
        print(f"Polling Data Extraction complete. Saved {len(cleaned_output)} records → {output_path}")
    elif extraction_type == "candidate_data":
        if polling_stations_json is None:

            cleaned_output = extract_candidate_data_tables_from_pdf(
                pdf_path=pdf_path,
                config=CONFIG[extraction_type],
            )
        else:
            with open(polling_stations_json, "r", encoding="utf-8") as f:
                polling_stations = json.load(f)
            cleaned_output = extract_candidate_data_tables_from_pdf(
                pdf_path=pdf_path,
                config=CONFIG[extraction_type],
                polling_stations=polling_stations,
            )
        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(cleaned_output, f, ensure_ascii=False, indent=4)
        print(f"Candidate Data Extraction complete. Saved {len(cleaned_output)} records → {output_path}")
    else:
        raise ValueError(f"Unsupported extraction type: {extraction_type}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Extract tables from a PDF into structured JSON."
    )
    parser.add_argument(
        "--pdf_path",
        type=str,
        required=True,
        help="Path to the PDF file to extract tables from.",
    )
    parser.add_argument(
        "--extraction_type",
        type=str,
        choices=["polling_station_data", "candidate_data"],
        required=True,
        help="Type of data extraction to perform.",
    )
    parser.add_argument(
        "--output_path",
        type=str,
        required=True,
        help="Path to save the extracted JSON data.",
    )
    parser.add_argument(
        "--polling_stations_json",
        type=str,
        required=False,
        help="Path to polling stations JSON file (required for candidate data extraction).",
    )
    args = parser.parse_args()

    extract_tables_from_pdf(
        pdf_path=args.pdf_path,
        extraction_type=args.extraction_type,
        output_path=args.output_path,
        polling_stations_json=args.polling_stations_json,
    )

