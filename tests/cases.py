from pathlib import Path

CASES_DIR = (Path(__file__).parent / "cases").resolve()
CACHE_DIR = (Path(__file__).parent / "cache").resolve()
CACHE_DIR.mkdir(exist_ok=True)

FEOX_PATH = Path(__file__).parent.parent / "target" / "debug" / "feox"

CASES = sorted(
    (p.parent.resolve() for p in Path("tests/cases").rglob("test.fe")),
    key=lambda p: str(p)
)
