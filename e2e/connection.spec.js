// exec-plans/active/001-connection-slice.md 완료 조건 1,2의 로컬 대역: 두 브라우저
// 컨텍스트가 같은 방에 들어와 서로의 fast 채널이 열리는가.
//
// 이 테스트는 "같은 머신 위의 두 브라우저"만 검증한다. 001의 진짜 산출물인
// "서로 다른 모바일 네트워크 간 연결 매트릭스"는 사람이 실제 폰으로 채운다
// (이 파일로 대체되지 않는다).
import { test, expect } from "@playwright/test";

const ROOM_HASH = "#r=E2ETEST&k=e2e-fixed-test-key-not-secret";

test("두 피어가 같은 방에서 서로의 fast 채널을 연다", async ({ browser }) => {
  const ctxA = await browser.newContext();
  const ctxB = await browser.newContext();
  const pageA = await ctxA.newPage();
  const pageB = await ctxB.newPage();

  // 진단용: 실패 원인이 "연결 안 됨"인지 "페이지 자체가 죽음"인지 구분한다.
  for (const [label, page] of [["A", pageA], ["B", pageB]]) {
    page.on("console", (msg) => console.log(`[${label} console] ${msg.type()}: ${msg.text()}`));
    page.on("pageerror", (err) => console.log(`[${label} pageerror] ${err}`));
  }

  try {
    await pageA.goto(`/${ROOM_HASH}`);
    await pageB.goto(`/${ROOM_HASH}`);

    // #debug-panel이 아예 없다면(초기화 실패로 main.js가 body를 통째로
    // 지웠다면) 아래 containText 타임아웃보다 먼저 여기서 드러나야 한다.
    await expect(pageA.locator("#debug-panel")).toHaveCount(1, { timeout: 10_000 });
    await expect(pageB.locator("#debug-panel")).toHaveCount(1, { timeout: 10_000 });

    // 디버그 패널이 "fast 1"을 보이면 두 방향 모두 unreliable 채널이 열린 것.
    await expect(pageA.locator("#debug-panel")).toContainText(/fast 1/, {
      timeout: 20_000,
    });
    await expect(pageB.locator("#debug-panel")).toContainText(/fast 1/, {
      timeout: 20_000,
    });
  } catch (err) {
    console.log("[A body]", await pageA.evaluate(() => document.body.innerText).catch((e) => String(e)));
    console.log("[B body]", await pageB.evaluate(() => document.body.innerText).catch((e) => String(e)));
    throw err;
  } finally {
    await ctxA.close();
    await ctxB.close();
  }
});
