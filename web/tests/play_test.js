// Drives the page in headless Chromium: plays full games by clicking legal steps.
const { chromium } = require("playwright");
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 860 } });
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
  await page.goto("file://" + require("path").resolve(__dirname, "../index.html"));
  await page.waitForTimeout(500);
  await page.screenshot({ path: process.argv[2] + "/start.png" });
  let games = 0, turns = 0;
  const t0 = Date.now();
  while (games < 3 && Date.now() - t0 < 240000) {
    const st = await page.evaluate(() => {
      const G = window.__mahbusa.G;
      return { turn: G.turn, dice: G.dice, over: G.over, banner: !document.getElementById("banner").hidden };
    });
    if (st.banner) { games++; if (games === 1) await page.screenshot({ path: process.argv[2] + "/gameover.png" }); await page.click("#bannerBtn"); await page.waitForTimeout(200); continue; }
    if (st.turn !== 0) { await page.waitForTimeout(150); continue; }
    if (!st.dice) { await page.click("#rollBtn"); await page.waitForTimeout(50); turns++; continue; }
    // play the first legal path step by step through the UI handlers (random path)
    // Regression check: the chances meter must not move while a move is half played.
    const bad = await page.evaluate(() => {
      const H = window.__mahbusa, G = H.G;
      const meter = () => document.getElementById("lgWin").textContent + "/" + document.getElementById("lgMars").textContent;
      const bad = [];
      if (!G.paths.length) return bad;
      const before = meter();
      const path = G.paths[Math.floor(Math.random() * G.paths.length)];
      for (const [a, b] of path.steps) {
        if (H.G.over) return bad;
        if (H.G.selected !== a) H.onPoint(a);
        H.onPoint(b);
        if (H.G.partial.length && H.G.partial.length < path.steps.length && meter() !== before) bad.push(`meter changed mid-move: ${before} -> ${meter()}`);
      }
      if (!H.G.over && H.G.dice) H.confirmMove();
      return bad;
    });
    errors.push(...bad);
    if (turns === 6) await page.screenshot({ path: process.argv[2] + "/midgame.png" });
    await page.waitForTimeout(50);
  }
  const final = await page.evaluate(() => ({ score: window.__mahbusa.G.score, log: window.__mahbusa.G.log.slice(-4).map((l) => l.text) }));
  console.log(JSON.stringify({ games, turns, errors, final }));
  await browser.close();
})();
