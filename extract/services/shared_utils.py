import re
import pandas as pd

from typing import Tuple, Optional
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

def is_subtotal_row(row: pd.Series) -> bool:
    """
    Detect if a row is a subtotal/grand total row.
    Looks for keywords in any of the row's values.
    """
    text = " ".join(str(v).lower() for v in row.values)
    return any(
        key in text for key in [
            "sub-total", "subtotal", "sub total",
            "total", "totals", "grand total"
        ]
    )
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