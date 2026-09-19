#!/usr/bin/env bash
# H7: 코드가 참조하는 assets/ 파일이 실제로 존재하고 asset-manifest.md에
# 출처·라이선스가 기록됐는지 대조한다. "보내고 안 그리는 건 방어가 아니듯,
# 매니페스트 없는 에셋은 존재하지 않는 것"이라는 원칙(AGENTS.md §6)의 CI 강제.
set -euo pipefail

MANIFEST="docs/generated/asset-manifest.md"
FAIL=0

REFS=$(grep -rhoE '"assets/[A-Za-z0-9_./-]+"' crates web 2>/dev/null | tr -d '"' | sort -u || true)

if [ -z "$REFS" ]; then
  echo "check_assets: 코드에서 assets/ 참조 없음 (통과)"
  exit 0
fi

while IFS= read -r path; do
  if [ ! -f "$path" ]; then
    echo "check_assets: $path 를 코드가 참조하지만 파일이 없습니다" >&2
    FAIL=1
  fi
  if ! grep -qF "$path" "$MANIFEST" 2>/dev/null; then
    echo "check_assets: $path 를 코드가 참조하지만 $MANIFEST 에 기록이 없습니다 (H7)" >&2
    FAIL=1
  fi
done <<< "$REFS"

exit $FAIL
