from cases import *
import subprocess
try:
    from tqdm import tqdm
    print = tqdm.write
except ImportError:
    tqdm = None

GREEN  = "\033[32m"
RED    = "\033[31m"
YELLOW = "\033[33m"
RESET  = "\033[0m"

for case in tqdm(CASES, unit=" tests") if tqdm else CASES:
    fe_file = case / f"test.fe"
    cache_file = CACHE_DIR / f"{case.relative_to(CASES_DIR)}.txt"

    result = subprocess.run(
        [FEOX_PATH, fe_file],
        capture_output=True, text=True
    )
    output = (result.stdout + result.stderr).strip()

    expected = cache_file.read_text()

    print("="*20)
    if output == expected:
        print(f"Test {case.relative_to(CASES_DIR)}: {GREEN}passed{RESET}")
    else:
        print(f"Test {case.relative_to(CASES_DIR)}: {RED}failed{RESET}")
        if len(output) < 50 and len(expected) < 50:
            print(f"    Output: {output}")
            print(f"    Expected: {expected}")