"""Query public OSV advisories for Cargo.lock registry packages; fail on unknowns.

Only public crate names and versions leave this machine. No application settings,
source, paths, credentials, or telemetry are sent. Requires Python 3.11+.
"""
import hashlib
import json
from pathlib import Path
import sys
import tomllib
from datetime import datetime, timezone
from urllib.request import Request, urlopen


def main():
    repo = Path(__file__).resolve().parent.parent
    lock_bytes = (repo / "source/Cargo.lock").read_bytes()
    packages = tomllib.loads(lock_bytes.decode("utf-8"))["package"]
    registry = [p for p in packages if p.get("source", "").startswith("registry+")]
    unsupported = [p for p in packages if p.get("source") and p not in registry]
    if not registry or unsupported:
        raise RuntimeError("No registry packages or unsupported non-registry dependencies")
    queries = [{"package": {"ecosystem": "crates.io", "name": p["name"]},
                "version": p["version"]} for p in registry]
    pending = list(enumerate(queries))
    findings = [[] for _ in registry]
    # Bound pagination: excessive/unexpected results are unknown, never a pass.
    for _ in range(20):
        request = Request("https://api.osv.dev/v1/querybatch",
                          data=json.dumps({"queries": [q for _, q in pending]}).encode(),
                          headers={"Content-Type": "application/json"}, method="POST")
        with urlopen(request, timeout=30) as response:
            payload = response.read(8 * 1024 * 1024 + 1)
        if len(payload) > 8 * 1024 * 1024:
            raise RuntimeError("OSV response exceeds bounded size")
        results = json.loads(payload)["results"]
        if len(results) != len(pending):
            raise RuntimeError("Incomplete OSV result count")
        next_page = []
        for (index, query), result in zip(pending, results):
            if not isinstance(result, dict) or result.get("error"):
                raise RuntimeError("OSV returned a malformed result or query error")
            findings[index].extend(v["id"] for v in result.get("vulns", []))
            if result.get("next_page_token"):
                next_page.append((index, {**query, "page_token": result["next_page_token"]}))
        pending = next_page
        if not pending:
            break
    if pending:
        raise RuntimeError("OSV pagination incomplete")
    report = {"checked_at_utc": datetime.now(timezone.utc).isoformat(),
              "lock_sha256": hashlib.sha256(lock_bytes).hexdigest(),
              "source": "https://api.osv.dev/v1/querybatch", "complete": True,
              "packages": [{"name": p["name"], "version": p["version"],
                            "advisories": sorted(set(ids))}
                           for p, ids in zip(registry, findings)]}
    output = repo / "source/target/dependency-audit.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    matched = sum(bool(ids) for ids in findings)
    print(f"OSV checked {len(registry)} locked crates; {matched} packages have advisories.")
    return 1 if matched else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"Dependency audit FAILED/UNKNOWN: {error}", file=sys.stderr)
        sys.exit(2)
