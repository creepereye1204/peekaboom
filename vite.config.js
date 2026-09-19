// repo-layout.md DD-006: base:'./'가 중요하다 — GitHub Pages 하위 경로 배포.
export default {
  root: "web",
  base: "./",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    target: "es2022",
  },
  server: {
    fs: {
      // pkg/ (wasm-bindgen 산출물)는 web/ 형제 디렉터리라 기본 허용 범위 밖.
      allow: [".."],
    },
  },
};
