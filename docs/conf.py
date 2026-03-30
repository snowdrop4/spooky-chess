from __future__ import annotations

from datetime import UTC, datetime
from pathlib import Path
import sys

DOCS_DIR = Path(__file__).resolve().parent
ROOT_DIR = DOCS_DIR.parent

sys.path.insert(0, str(DOCS_DIR / "_ext"))

from generate_stub_reference import generate_stub_reference

generate_stub_reference(ROOT_DIR / "spooky_chess.pyi", DOCS_DIR / "_generated" / "python_api_reference.rst")

project = "spooky_chess"
author = "snowdrop4"
copyright = f"{datetime.now(UTC).year}, {author}"  # noqa: A001

extensions: list[str] = []
exclude_patterns = ["_build", "_generated", "Thumbs.db", ".DS_Store"]
html_static_path: list[str] = []
root_doc = "index"
nitpicky = False
