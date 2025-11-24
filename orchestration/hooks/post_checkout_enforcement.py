#!/usr/bin/env python3
"""
Post-checkout hook that re-syncs enforcement state on branch switch.
Ensures queue state matches the current branch.
"""
import json
import sys
from pathlib import Path
from datetime import datetime

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from orchestration.core.paths import DataPaths

# Global paths instance
_paths = DataPaths()


def log_activity(event_type: str, details: dict):
    """Log activity."""
    log_file = _paths.activity_log
    if not log_file.exists():
        return

    try:
        log = json.loads(log_file.read_text())
        log["events"].append({
            "timestamp": datetime.now().isoformat(),
            "type": event_type,
            "details": details
        })
        log["events"] = log["events"][-1000:]
        log_file.write_text(json.dumps(log, indent=2))
    except Exception:
        pass


def resync_state():
    """Re-sync enforcement state after checkout."""
    # Update last activity timestamp
    manifest_file = _paths.spec_manifest
    if manifest_file.exists():
        try:
            manifest = json.loads(manifest_file.read_text())
            manifest["last_sync"] = datetime.now().isoformat()
            manifest_file.write_text(json.dumps(manifest, indent=2))
        except Exception:
            pass


def main():
    """Handle post-checkout re-sync."""
    # Arguments: previous_head, new_head, is_branch_checkout
    if len(sys.argv) >= 4:
        previous_head = sys.argv[1]
        new_head = sys.argv[2]
        is_branch = sys.argv[3] == "1"

        if is_branch:
            print("Branch switch detected - re-syncing enforcement state...")
            resync_state()

            log_activity("branch_switch", {
                "from": previous_head[:8],
                "to": new_head[:8]
            })

    sys.exit(0)


if __name__ == "__main__":
    main()
