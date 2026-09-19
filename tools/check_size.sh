#!/usr/bin/env bash
# H5: WASM 번들(gzip) <= 1.5MB, 최초 인터랙션까지 3초 예산의 기반.
# 사용: tools/check_size.sh [wasm 경로]
set -euo pipefail

WASM_PATH="${1:-pkg/peekaboom_app_bg.wasm}"
LIMIT_BYTES=$((1536 * 1024)) # 1.5 MiB

if [ ! -f "$WASM_PATH" ]; then
  echo "check_size: $WASM_PATH 없음. 먼저 wasm-bindgen을 돌렸는지 확인하세요." >&2
  exit 1
fi

GZIP_SIZE=$(gzip -9 -c "$WASM_PATH" | wc -c)

echo "check_size: $WASM_PATH gzip 크기 = ${GZIP_SIZE} bytes (한도 ${LIMIT_BYTES} bytes, H5)"

if [ "$GZIP_SIZE" -gt "$LIMIT_BYTES" ]; then
  echo "check_size: H5 위반 — 번들이 1.5MB(gzip)를 초과했습니다." >&2
  exit 1
fi
