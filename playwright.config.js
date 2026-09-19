// repo-layout.md CI 13단계: 2피어 E2E. 한 머신 안의 두 브라우저 컨텍스트가
// 실제 공개 릴레이(Nostr)로 시그널링한다 — CB-5(공개 인프라는 언젠가 죽는다)의
// 위험을 CI에서도 그대로 안는다는 뜻이다. 릴레이가 죽으면 이 테스트도 실패한다.
export default {
  testDir: "./e2e",
  timeout: 30_000,
  retries: 1,
  webServer: {
    command: "npm run preview -- --port 4173 --strictPort",
    port: 4173,
    reuseExistingServer: !process.env.CI,
  },
  use: {
    baseURL: "http://localhost:4173",
  },
};
