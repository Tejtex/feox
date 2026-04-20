import subprocess
from cases import *
try:
    from tqdm import tqdm
    print = tqdm.write
except ImportError:
    tqdm = None

for case in tqdm(CASES, unit=" tests") if tqdm else CASES:
    py_file = case / "test.py"
    cache_file = CACHE_DIR / f"{case.relative_to(CASES_DIR)}.txt"

    cache_file.parent.mkdir(exist_ok=True)

    result = subprocess.run(
        ["python3", py_file],
        capture_output=True, text=True
    )
    output = (result.stdout + result.stderr).strip()

    cache_file.write_text(output)

    print(f"Generated output for {case.relative_to(CASES_DIR)}")