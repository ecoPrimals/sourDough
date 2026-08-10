#!/usr/bin/env bash
# Forgejo post-receive hook for sovereign CI
#
# Install: symlink or copy to each primal repo's hooks/post-receive on golgi
#   ln -s /opt/sourdough/ci/forgejo-post-receive-hook.sh /data/gitea/repositories/ecoPrimals/<primal>.git/hooks/post-receive
#
# Requires: sourdough binary in PATH (e.g., /usr/local/bin/sourdough)
#
# Behavior:
#   - On push to main: checks out worktree, runs `sourdough ci`
#   - Reports pass/fail to stdout (visible in push output)
#   - Non-blocking: push always succeeds (CI is advisory, not gating)
#   - Results logged to /var/log/sourdough-ci/<primal>-<timestamp>.json
#
# To make CI gating (reject push on failure): change exit 0 to exit $STATUS

set -euo pipefail

SOURDOUGH="${SOURDOUGH_BIN:-sourdough}"
LOG_DIR="${SOURDOUGH_CI_LOG_DIR:-/var/log/sourdough-ci}"
WORK_DIR="${SOURDOUGH_CI_WORK_DIR:-/tmp/sourdough-ci}"

mkdir -p "$LOG_DIR" "$WORK_DIR"

while read -r OLD_REV NEW_REV REF_NAME; do
    # Only run on main branch pushes
    if [[ "$REF_NAME" != "refs/heads/main" ]]; then
        continue
    fi

    # Extract repo name from GIT_DIR (Forgejo sets this)
    REPO_PATH="${GIT_DIR:-.}"
    REPO_NAME=$(basename "$REPO_PATH" .git)
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)

    echo "━━━ sourDough CI ━━━ $REPO_NAME @ ${NEW_REV:0:7}"

    # Checkout to temporary worktree
    CHECKOUT="$WORK_DIR/$REPO_NAME-$TIMESTAMP"
    git --git-dir="$REPO_PATH" worktree add --detach "$CHECKOUT" "$NEW_REV" 2>/dev/null || {
        echo "  ⚠ worktree checkout failed — skipping CI"
        continue
    }

    # Run sourDough CI
    LOG_FILE="$LOG_DIR/${REPO_NAME}_${TIMESTAMP}.json"

    if "$SOURDOUGH" ci "$CHECKOUT" --json > "$LOG_FILE" 2>/dev/null; then
        echo "  ✓ CI PASS"
        STATUS=0
    else
        echo "  ✗ CI FAIL — see $LOG_FILE"
        STATUS=1
    fi

    # Cleanup worktree
    git --git-dir="$REPO_PATH" worktree remove --force "$CHECKOUT" 2>/dev/null || true

    # Log summary
    echo "  → $LOG_FILE"

done

# Advisory mode: always allow push (change to `exit $STATUS` for gating)
exit 0
