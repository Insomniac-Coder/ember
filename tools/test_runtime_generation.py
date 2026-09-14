"""Regression tests for the ``[RT-5]`` runtime generator."""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import generate_runtime  # noqa: E402


def main() -> int:
    assert generate_runtime.branding_prefix() == "ember"
    header, source = generate_runtime.generated_contents("aurora")
    assert "#define AURORA_SYMBOL_PREFIX aurora" in header
    assert '#include "aurora_rt.h"' in source
    assert "aurora_alloc" in header and "aurora_alloc" in source
    assert "ember_alloc" not in header and "ember_alloc" not in source
    assert "typedef struct aurora_obj_header" in header
    assert "uint32_t strong;" in header
    assert "uint32_t weak;" in header
    assert "uint32_t access;" in header
    assert "uint32_t flags;" in header
    assert "const aurora_type_info* ti;" in header
    assert "struct aurora_type_info" in header
    assert "#define AURORA_TI_SYNC UINT32_C(1)" in header
    assert "aurora_rt_deinit(object)" in source
    assert "ti->drop_fields(object)" in source
    assert "aurora_weak_upgrade" in header and "aurora_weak_upgrade" in source
    assert "object_panic_resurrection" in source
    assert "aurora_panic_exclusivity" in header and "aurora_panic_exclusivity" in source

    with tempfile.TemporaryDirectory() as directory:
        out_dir = Path(directory) / "runtime"
        paths = generate_runtime.write_outputs(out_dir, "aurora")
        assert all(path.exists() for path in paths)
        assert generate_runtime.check_outputs(out_dir, "aurora")

    assert generate_runtime.check_outputs(generate_runtime.DEFAULT_OUT, "ember")
    print("runtime generator tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
