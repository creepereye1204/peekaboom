// 방 코드 / URL / localStorage. `#r=CODE&k=KEY` 파싱·생성 (SECURITY.md §5).
// URL 프래그먼트는 HTTP 요청에 안 실리므로 GitHub 서버 로그에도 안 남는다.

const CODE_CHARS = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // 헷갈리는 0/O, 1/I 제외
const CODE_LEN = 6;

function randomCode() {
  const bytes = crypto.getRandomValues(new Uint8Array(CODE_LEN));
  return Array.from(bytes, (b) => CODE_CHARS[b % CODE_CHARS.length]).join("");
}

function randomKeyBase64Url() {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  let bin = "";
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function parseFragment() {
  const params = new URLSearchParams(location.hash.replace(/^#/, ""));
  const code = params.get("r");
  const key = params.get("k");
  if (code && key) return { code, key };
  return null;
}

/**
 * 현재 URL에서 방 정보를 읽고, 없으면 새로 만들어 URL에 반영한다.
 * @returns {{code: string, key: string, isNew: boolean}}
 */
export function getOrCreateRoom() {
  const existing = parseFragment();
  if (existing) return { ...existing, isNew: false };

  const code = randomCode();
  const key = randomKeyBase64Url();
  const newHash = `#r=${code}&k=${key}`;
  history.replaceState(null, "", newHash);
  return { code, key, isNew: true };
}

export function shareUrl() {
  return location.href;
}

/** `navigator.share` 우선, 실패/미지원 시 클립보드 복사로 폴백. */
export async function shareRoom() {
  const url = shareUrl();
  if (navigator.share) {
    try {
      await navigator.share({ title: "PEEKABOOM", url });
      return "shared";
    } catch (err) {
      if (err && err.name === "AbortError") return "cancelled";
      // 공유 API가 있지만 실패한 경우 클립보드로 폴백
    }
  }
  try {
    await navigator.clipboard.writeText(url);
    return "copied";
  } catch {
    return "failed";
  }
}
