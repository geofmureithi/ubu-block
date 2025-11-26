import re
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