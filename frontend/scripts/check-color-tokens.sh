#!/usr/bin/env bash
# check-color-tokens.sh — design-token guardrail for DDDUI renderer.
#
# Goal: prevent NEW raw color literals (#RRGGBB / #RRGGBBAA and rgba()) that
# bypass the design-token system in UI chrome.
#
# Accepted (never flagged):
#   - var(--x, #fallback)   token fallbacks with a hex safety net
#   - linear/radial-gradient decorative colors
#   - rgba(...) alpha / shadow colors (tracked separately as debt)
#
# Excluded by path (intentional, non-chrome, or feature-specific colors):
#   token definitions, decorative theme presets, syntax-highlight palette,
#   team avatar palette, third-party brand colors, Landing decorative accents,
#   CSS-theme editor preview, categorical badge palette, input focus ring,
#   HTML inspect overlay, ACP protocol status, thought gradient, preview
#   dropdowns/context-menu, Arco ConfigProvider, brand highlight, status dot.
#
# Baseline mode: on first run (`--init`) every currently-offending file is
# recorded as accepted legacy debt. Later runs fail ONLY on NEW offenders
# (files not in the baseline), so cleaned files stay clean and new code must
# use tokens.
#
# Usage:
#   scripts/check-color-tokens.sh            # check, fail on new offenders
#   scripts/check-color-tokens.sh --init     # (re)create the baseline
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RENDERER="$SCRIPT_DIR/../packages/desktop/src/renderer"
BASELINE="$SCRIPT_DIR/.color-token-baseline.txt"
cd "$RENDERER"

# Files/dirs whose raw colors are intentional and excluded from scanning.
EXTRA_GLOBS=(
  -g '!**/styles/themes/**'
  -g '!**/styles/colors.ts'
  -g '!**/styles/arco-override.css'
  -g '!**/presets/**'
  -g '!**/markdownHighlightStyle.ts'
  -g '!**/teamMemberColors.ts'
  -g '!**/AddPlatformModal.tsx'
  -g '!**/Landing/index.tsx'
  -g '!**/CssThemeSettings.tsx'
  -g '!**/SkillsSettings/**'
  -g '!**/useInputFocusRing.ts'
  -g '!**/htmlInspectScript.ts'
  -g '!**/HTMLViewer.tsx'
  -g '!**/useAcpMessage.ts'
  -g '!**/ThoughtDisplay.tsx'
  -g '!**/PreviewHistoryDropdown.tsx'
  -g '!**/PreviewContextMenu.tsx'
  -g '!**/main.tsx'
  -g '!**/components/layout/Layout.tsx'
  -g '!**/GroupedHistory/ConversationRow.tsx'
)

# Find true raw-hex offenders (skip fallback / gradient / rgba lines), as files.
scan_offenders() {
  rg -nH -Ii -g '*.ts' -g '*.tsx' -g '*.css' "${EXTRA_GLOBS[@]}" \
    -e '#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?\b' . 2>/dev/null \
    | rg -v 'var\(-\-' \
    | rg -v -i 'gradient' \
    | rg -v 'rgba\(' \
    | sed -E 's/^([^:]+):[0-9]+:.*/\1/' \
    | sed 's|^\./||' \
    | sort -u
}

if [ "${1:-}" = '--init' ]; then
  scan_offenders > "$BASELINE"
  echo "✅ Baseline written: $(grep -c . "$BASELINE") legacy files recorded in"
  echo "   $BASELINE"
  exit 0
fi

if [ ! -f "$BASELINE" ]; then
  echo "⚠️  No baseline found. Run: scripts/check-color-tokens.sh --init"
  echo "    (records current offenders as accepted legacy debt)"
  exit 0
fi

CURRENT="$(scan_offenders)"
BASE="$(sort -u "$BASELINE")"

NEW="$(comm -13 <(printf '%s\n' "$BASE") <(printf '%s\n' "$CURRENT"))"
REMOVED="$(comm -23 <(printf '%s\n' "$BASE") <(printf '%s\n' "$CURRENT"))"

if [ -n "$REMOVED" ]; then
  echo "🎉 Debt paid down (no longer offending):"
  echo "$REMOVED" | sed 's/^/   - /'
fi

if [ -n "$NEW" ]; then
  echo "❌ NEW raw color literals bypassing design tokens:"
  echo "$NEW" | sed 's/^/   - /'
  echo ""
  echo "If intentional, re-run: scripts/check-color-tokens.sh --init"
  exit 1
else
  echo "✅ No new raw color literals ($(printf '%s\n' "$CURRENT" | grep -c .) legacy files tracked, unchanged)."
fi
