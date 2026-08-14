"""数字海洋生命集合馆：北斗浮蚕 + yuruyurau / Matlab digital life 系列。"""

INDEX_HTML = r"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>数字海洋生命 · 集合馆</title>
  <style>
    :root {
      --bg:#050505; --fg:#e8ffe8; --accent:#ffe08a; --cyan:#7fffd4;
      --glass: rgba(6, 10, 12, 0.58); --line: rgba(255,255,255,0.14);
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    html, body { height: 100%; width: 100%; overflow: hidden; background: #050505; color: var(--fg);
      font-family: "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif; }
    canvas { position: fixed; inset: 0; width: 100%; height: 100%; display: block; }
    body.ocean canvas { cursor: crosshair; }
    .scan {
      position: fixed; inset: 0; z-index: 1; pointer-events: none; display: none;
      background: repeating-linear-gradient(
        to bottom, transparent 0, transparent 3px, rgba(0, 8, 12, 0.16) 3px, rgba(0, 8, 12, 0.16) 4px
      );
    }
    body.ocean .scan { display: block; }
    body.saver .topbar,
    body.saver .info,
    body.saver .formula,
    body.saver .strip { display: none !important; }
    body.saver, body.saver canvas { cursor: none; user-select: none; }
    body.ocean:not(.saver) canvas.legend-hit { cursor: pointer; }
    .topbar {
      position: fixed; top: 0; left: 0; right: 0; z-index: 3;
      display: flex; align-items: center; justify-content: space-between; gap: 1rem;
      padding: 0.65rem 1rem 0.8rem;
      background: linear-gradient(180deg, rgba(0,0,0,0.78), rgba(0,0,0,0.2) 75%, transparent);
      pointer-events: none;
    }
    .topbar > * { pointer-events: auto; }
    .brand { display: flex; align-items: center; gap: 0.7rem; min-width: 0; }
    .mark {
      width: 9px; height: 9px; border-radius: 50%; flex: 0 0 auto;
      background: var(--accent); box-shadow: 0 0 12px var(--accent);
    }
    body.ocean .mark { background: var(--cyan); box-shadow: 0 0 12px var(--cyan); }
    .brand h1 {
      font-size: 0.92rem; font-weight: 650; letter-spacing: 0.18em;
      color: var(--accent); white-space: nowrap;
    }
    body.ocean .brand h1 { color: var(--cyan); }
    .brand p {
      margin-top: 0.12rem; font-size: 0.72rem; color: rgba(180,210,180,0.78);
      white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: min(38vw, 420px);
    }
    .tools { display: flex; gap: 0.38rem; flex-shrink: 0; flex-wrap: wrap; justify-content: flex-end; }
    .info {
      position: fixed; top: 4.35rem; left: 1rem; z-index: 3; pointer-events: none;
      width: min(300px, calc(100vw - 2rem));
      padding: 0.8rem 0.95rem 0.9rem;
      background: var(--glass); border: 1px solid var(--line); border-radius: 16px;
      backdrop-filter: blur(14px); -webkit-backdrop-filter: blur(14px);
      box-shadow: 0 12px 40px rgba(0,0,0,0.35);
    }
    .info .kicker {
      font-size: 0.66rem; letter-spacing: 0.18em; text-transform: uppercase;
      color: var(--accent); margin-bottom: 0.28rem;
    }
    body.ocean .info { display: none; }
    .info h2 { font-size: 1.08rem; font-weight: 600; color: #fff; line-height: 1.35; }
    .info .blurb { margin-top: 0.38rem; font-size: 0.8rem; color: #c5e8c5; line-height: 1.5; }
    .info .blurb.en { margin-top: 0.2rem; font-size: 0.74rem; color: #8fb8b0; }
    .formula {
      position: fixed; right: 1rem; bottom: 6.4rem; z-index: 3; pointer-events: none;
      max-width: min(440px, 44vw);
      padding: 0.7rem 0.85rem;
      background: var(--glass); border: 1px solid var(--line); border-radius: 14px;
      backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px);
      text-align: right; font-family: Georgia, "Times New Roman", serif;
      font-size: 0.72rem; line-height: 1.6; color: rgba(255,255,255,0.84);
      word-break: break-word;
    }
    .formula[hidden] { display: none !important; }
    .strip {
      position: fixed; bottom: 0; left: 0; right: 0; z-index: 3;
      display: flex; gap: 0.5rem; overflow-x: auto;
      padding: 0.7rem 1rem 0.9rem;
      background: linear-gradient(0deg, rgba(0,0,0,0.88), rgba(0,0,0,0.4) 55%, transparent);
      scrollbar-width: thin;
    }
    .card {
      flex: 0 0 auto; width: 112px; border: 1px solid var(--line);
      background: rgba(10,12,14,0.78); color: var(--fg); border-radius: 12px;
      padding: 0.55rem 0.45rem; cursor: pointer; text-align: center;
      transition: transform .16s ease, border-color .16s ease, background .16s ease;
    }
    .card strong { display: block; font-size: 0.82rem; }
    .card span { display: block; margin-top: 0.18rem; font-size: 0.66rem; color: #8fbf8f; }
    .card:hover { transform: translateY(-2px); border-color: rgba(255,224,138,0.45); }
    .card.on { background: rgba(40,36,20,0.92); border-color: var(--accent); color: var(--accent); }
    .card.ocean { width: 122px; border-color: rgba(62,230,255,0.4); color: #b8fff4; }
    .card.ocean.on { background: rgba(0,36,48,0.95); border-color: #3ee6ff;
      box-shadow: 0 0 16px rgba(0,255,220,0.22); color: #7fffd4; }
    a.btn, button {
      border: 0; border-radius: 999px; padding: 0.4rem 0.85rem; font-size: 0.8rem;
      font-weight: 600; cursor: pointer; text-decoration: none; display: inline-block;
      transition: transform .15s ease, filter .15s ease;
    }
    a.btn:hover, button:hover { transform: translateY(-1px); filter: brightness(1.08); }
    button { color: #163018; background: linear-gradient(180deg, #ffe9a8, #e7c04a); }
    #btnOcean { color: #042018; background: linear-gradient(180deg, #b8fff4, #3ee6ff); }
    #btnShuffle { color: #e8ffe8; background: rgba(255,255,255,0.1); border: 1px solid rgba(62,230,255,0.4); }
    a.btn { color: #e8ffe8; background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2); }
    @media (max-width: 740px) {
      .brand p { display: none; }
      .info { width: calc(100vw - 2rem); }
      .formula { left: 1rem; right: 1rem; max-width: none; text-align: left; bottom: 6.8rem; }
      a.btn, button { padding: 0.36rem 0.7rem; font-size: 0.74rem; }
    }
  </style>
</head>
<body>
  <canvas id="cv"></canvas>
  <div class="scan"></div>
  <header class="topbar">
    <div class="brand">
      <span class="mark"></span>
      <div>
        <h1 id="title">数字生命 · Digital Life</h1>
        <p id="sub">集合馆 · Gallery</p>
      </div>
    </div>
    <div class="tools">
      <button type="button" id="btnOcean">赛博海洋 · Ocean</button>
      <button type="button" id="btnShuffle" style="display:none">重新随机 · Shuffle</button>
      <button type="button" id="btnPause">暂停 · Pause</button>
      <a class="btn" id="btnLesson" href="/lesson">分步讲解 · Lesson</a>
    </div>
  </header>
  <aside class="info">
    <div class="kicker" id="kicker"></div>
    <h2 id="name"></h2>
    <p class="blurb" id="blurb"></p>
    <p class="blurb en" id="blurbEn"></p>
  </aside>
  <aside class="formula" id="formula" hidden></aside>
  <nav class="strip" id="strip"></nav>
  <script>
(function () {
  "use strict";
  var WALLPAPER = /(?:[?&]wallpaper=1\b)/.test(location.search)
    || document.body.getAttribute("data-mode") === "wallpaper";
  var SAVER = !WALLPAPER && (
    /(?:[?&]saver=1\b)/.test(location.search)
    || /\/screensaver(?:\.html)?$/i.test(location.pathname)
    || document.body.getAttribute("data-mode") === "saver"
  );
  if (SAVER || WALLPAPER) {
    document.body.classList.add("saver", "ocean");
    document.title = SAVER ? "赛博海洋馆 · Cyber Ocean Screensaver" : "赛博海洋馆 · Cyber Ocean";
    document.addEventListener("contextmenu", function (e) { e.preventDefault(); });
    function goFull() {
      var el = document.documentElement;
      var fn = el.requestFullscreen || el.webkitRequestFullscreen || el.mozRequestFullScreen;
      if (fn) { try { fn.call(el); } catch (err) {} }
    }
    goFull();
    setTimeout(goFull, 250);
    setTimeout(goFull, 1200);
  }
  var SHOT = /(?:[?&]shot=1\b)/.test(location.search);
  var seedMatch = location.search.match(/[?&]seed=(\d+)/);
  var FIXED_SEED = seedMatch ? (parseInt(seedMatch[1], 10) >>> 0) : null;
  var MAX = 42000;
  var FILL_STEP = 1;
  var outX = new Float32Array(MAX);
  var outY = new Float32Array(MAX);
  var count = 0;

  function finite(x, y) { return isFinite(x) && isFinite(y); }

  function fillFucan(t) {
    var n = 0, N = 22000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 100, y = (i / 100) | 0;
      var k = x / 4 - 12.5, e = y / 9 + 6, o = Math.sqrt(k * k + e * e) / 9;
      if (Math.abs(k) < 1e-9 || o < 1e-9 || Math.abs(Math.cos(y / 2)) < 0.015) continue;
      var ht = 0.5 * Math.tan(y / 2);
      if (!isFinite(ht) || Math.abs(ht) > 60) continue;
      var c = o / 2 + e / 2 - t / 4;
      var q = (3 / k) * (ht + Math.cos(y)) + k * (5 / o + o * Math.sin(y) * Math.sin(e + 4 * o - t));
      var X = q + 40 * Math.cos(c);
      var Y = q * Math.sin(c) - (o * k * k) / 6 + 12 * e * o;
      if (!finite(X, Y)) continue;
      var scale = 0.82;
      outX[n] = 200 + X * scale;
      outY[n] = 28 + (Y - 50) * scale;
      n++;
    }
    return n;
  }

  function fillYouyan(t) {
    var n = 0, N = 18000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 100, y = (i / 100) | 0;
      var k = x / 4 - 12.5, e = y / 9 + 5, o = Math.sqrt(k * k + e * e) / 9;
      if (Math.abs(k) < 1e-6) continue;
      var q = x + 99 + Math.tan(1 / k) + o * k * (Math.cos(e * 9) / 4 + Math.cos(y / 2)) * Math.sin(o * 4 - t);
      var c = o * e / 30 - t / 8;
      var X = q * 0.7 * Math.sin(c) + 9 * Math.cos(y / 19 + t) + 200;
      var Y = 200 + q / 2 * Math.cos(c);
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillJichong(t) {
    var n = 0, N = 9000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i, y = i / 235, e = y / 8 - 13;
      var k = (4 + Math.sin(y * 2 - t) * 3) * Math.cos(x / 29);
      if (Math.abs(k) < 1e-6) continue;
      var d = Math.sqrt(k * k + e * e);
      var q = 3 * Math.sin(k * 2) + 0.3 / k + Math.sin(y / 25) * k * (9 + 4 * Math.sin(e * 9 - d * 3 + t * 2));
      var X = q + 30 * Math.cos(d - t) + 200;
      var Y = 620 - q * Math.sin(d - t) - d * 39;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillJelly(t) {
    var n = 0, N = 10000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 200, y = i / 43;
      var k = 5 * Math.cos(x / 14) * Math.cos(y / 30);
      var e = y / 8 - 13;
      var d = (k * k + e * e) / 59 + 4;
      var a = Math.atan2(k, e);
      var q = 60 - 3 * Math.sin(a * e) + k * (3 + 4 / d * Math.sin(d * d - t * 2));
      var c = d / 2 + e / 99 - t / 18;
      var X = q * Math.sin(c) + 200;
      var Y = (q + d * 9) * Math.cos(c) + 200;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillNebula(t) {
    var n = 0, N = 20000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 200, y = i / 200;
      var k = x / 8 - 12.5, e = y / 8 - 12.5;
      var o = (k * k + e * e) / 169;
      var d = 0.5 + 5 * Math.cos(o);
      var X = x + d * k * Math.sin(d * 2 + o + t) + e * Math.cos(e + t) + 100;
      var Y = y / 4 - o * 135 + d * 6 * Math.cos(d * 3 + o * 9 + t) + 275;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillLantern(t) {
    var n = 0, N = 10000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 200, y = i / 55;
      var k = 9 * Math.cos(x / 8), e = y / 8 - 12.5;
      var d = (k * k + e * e) / 99 + Math.sin(t) / 6 + 0.5;
      if (Math.abs(d) < 1e-6) continue;
      var q = 99 - e * Math.sin(Math.atan2(k, e) * 7) / d + k * (3 + Math.cos(d * d - t) * 2);
      var c = d / 2 + e / 69 - t / 16;
      var X = q * Math.sin(c) + 200;
      var Y = (q + 19 * d) * Math.cos(c) + 200;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillFeather(t) {
    var n = 0, N = 9000;
    for (var i = 1; i <= N; i += FILL_STEP) {
      var y = i / 790, k;
      if (y < 5) k = 6 + Math.sin((Math.floor(y) ^ 1)) * 6;
      else k = 4 + Math.cos(y);
      var cs = Math.cos(i + t / 4);
      var d = Math.sqrt((k * cs) * (k * cs) + (y / 3 - 13) * (y / 3 - 13));
      var q = y * k * cs / 5 * (2 + Math.sin(d * 2 + y - t * 4));
      var c = d / 3 - t / 2 + (i % 2);
      var X = q + 90 * Math.cos(c) + 200;
      var Y = 400 - (q * Math.sin(c) + d * 29 - 170);
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillTentacle(t) {
    var n = 0, N = 9000;
    for (var i = 1; i <= N; i += FILL_STEP) {
      var y = i / 345, x = y;
      if (y < 11) x = 6 + Math.sin((Math.floor(x) ^ 8)) * 6;
      else x = x / 5 + Math.cos(x / 2);
      var e = y / 7 - 13;
      var k = x * Math.cos(i - t / 4);
      var d = Math.sqrt(k * k + e * e) + Math.sin(e / 4 + t) / 2;
      if (Math.abs(d) < 1e-6) continue;
      var q = y * k / d * (3 + Math.sin(d * 2 + y / 2 - t * 4));
      var c = d / 2 + 1 - t / 2;
      var X = q + 60 * Math.cos(c) + 200;
      var Y = 400 - (q * Math.sin(c) + d * 29 - 170);
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillFlower6(t) {
    var n = 0, N = 5000, copies = 6, ang = Math.PI / 3;
    for (var i = 1; i <= N; i += FILL_STEP) {
      var k = (i % 25) - 12, e = i / 800;
      var d = 7 * Math.cos(Math.sqrt(k * k + e * e) / 3 + t / 2);
      var bx = k * 4 + d * k * Math.sin(d + e / 9 + t);
      var by = e * 2 - d * 9 - d * 9 * Math.cos(d + t);
      for (var j = 0; j < copies; j++) {
        var a = j * ang, ca = Math.cos(a), sa = Math.sin(a);
        var X = ca * bx - sa * by + 200;
        var Y = sa * bx + ca * by + 200;
        if (!finite(X, Y) || n >= MAX) continue;
        outX[n] = X; outY[n] = Y; n++;
      }
    }
    return n;
  }

  function fillWheel(t) {
    var n = 0, N = 3500, copies = 14, ang = Math.PI / 7;
    for (var i = 1; i <= N; i += FILL_STEP) {
      var k = (i % 50) - 25, e = i / 1100;
      var d = 5 * Math.cos(Math.sqrt(k * k + e * e) - t + (i % 2));
      if (Math.abs(d) < 1e-6) continue;
      var bx = k + k * d / 6 * Math.sin(d + e / 3 + t);
      var by = 90 + e * d - e / d * 2 * Math.cos(d + t);
      for (var j = 0; j < copies; j++) {
        var a = j * ang, ca = Math.cos(a), sa = Math.sin(a);
        var X = ca * bx - sa * by + 200;
        var Y = sa * bx + ca * by + 200;
        if (!finite(X, Y) || n >= MAX) continue;
        outX[n] = X; outY[n] = Y; n++;
      }
    }
    return n;
  }

  function fillSpiral(t) {
    var n = 0, N = 12000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 120, y = (i / 120) | 0;
      var k = x / 5 - 12, e = y / 8 - 8;
      var o = Math.sqrt(k * k + e * e) / 8;
      var c = o * 1.15 + t / 5;
      var q = 22 + 10 * Math.sin(e * 0.8 + t) + k * (1.6 + 0.35 * Math.sin(3 * o - t));
      var X = q * Math.cos(c) + 10 * Math.sin(e * 2 + t) + 200;
      var Y = q * Math.sin(c) * 0.88 + 200;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillComb(t) {
    var n = 0, N = 10000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 180, y = i / 50;
      var k = 7 * Math.cos(x / 10) * Math.cos(y / 35);
      var e = y / 8 - 12;
      var d = (k * k + e * e) / 70 + 3;
      if (Math.abs(d) < 1e-6) continue;
      var a = Math.atan2(k, e);
      var q = 48 - 4 * Math.sin(a * 4) + k * (2.2 + 3 / d * Math.sin(d * d - t));
      var c = d / 2.4 + e / 85 - t / 14;
      var X = q * Math.sin(c) + 200;
      var Y = (q + 7 * d) * Math.cos(c) * 0.78 + 12 * Math.sin(x / 18 + t) + 210;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillSawEel(t) {
    var n = 0, N = 9000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i, y = i / 210, e = y / 9 - 12;
      var k = (3.5 + Math.sin(y * 1.6 - t) * 2.4) * Math.cos(x / 22);
      if (Math.abs(k) < 1e-6) continue;
      var d = Math.sqrt(k * k + e * e);
      var q = 2.2 * Math.sin(k * 3) + 0.25 / k + Math.sin(y / 18) * k * (7 + 3 * Math.sin(e * 6 - d * 2 + t * 2));
      var X = q + 24 * Math.cos(d * 0.7 - t) + 200;
      var Y = 560 - q * Math.sin(d * 0.7 - t) - d * 32;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillStar8(t) {
    var n = 0, N = 4000, copies = 8, ang = Math.PI / 4;
    for (var i = 1; i <= N; i += FILL_STEP) {
      var k = (i % 20) - 10, e = i / 900;
      var d = 6 * Math.cos(Math.sqrt(k * k + e * e) / 4 + t / 3);
      var bx = 5 * k + d * k * Math.sin(d + t);
      var by = 2.5 * e - 8 * d * Math.cos(d + e / 8 + t);
      for (var j = 0; j < copies; j++) {
        var a = j * ang, ca = Math.cos(a), sa = Math.sin(a);
        var X = ca * bx - sa * by + 200;
        var Y = sa * bx + ca * by + 200;
        if (!finite(X, Y) || n >= MAX) continue;
        outX[n] = X; outY[n] = Y; n++;
      }
    }
    return n;
  }

  function fillShrimp(t) {
    var n = 0, N = 14000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 100, y = (i / 100) | 0;
      var k = x / 4 - 12.5, e = y / 8 + 3.5;
      var o = Math.sqrt(k * k + e * e) / 8;
      if (Math.abs(k) < 1e-6 || o < 1e-6) continue;
      var q = 55 + 10 * Math.sin(k * 0.8) + k * (2.2 + 0.55 * o * Math.sin(y * 0.7 - t));
      var c = o / 3.2 + e / 22 - t / 9;
      var X = q * 0.5 * Math.sin(c) + 7 * Math.cos(y / 16 + t) + 200;
      var Y = 200 + q * 0.38 * Math.cos(c) + 6 * Math.sin(k + t * 0.6);
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillVortex(t) {
    var n = 0, N = 14000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = (i % 200) - 100, y = (i / 200) - 35;
      var r = Math.sqrt(x * x + y * y) / 38;
      var th = Math.atan2(y, x);
      var R = 62 + 22 * Math.sin(3 * th + t) + 10 * Math.sin(r * 5 - t * 2);
      var X = R * Math.cos(th + r * 0.45 + t / 7) + 200;
      var Y = R * Math.sin(th + r * 0.45 + t / 7) * 0.9 + 200;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  function fillAngel(t) {
    var n = 0, N = 10000;
    for (var i = 0; i < N; i += FILL_STEP) {
      var x = i % 160, y = i / 65;
      var k = 6.5 * Math.cos(x / 12) * Math.cos(y / 38);
      var e = y / 9 - 11;
      var d = (k * k + e * e) / 68 + 2.4;
      if (Math.abs(d) < 1e-6) continue;
      var a = Math.atan2(k, e);
      var q = 38 - 5 * Math.sin(a * 3) + k * (2 + 3.2 / d * Math.sin(d * 2.2 - t));
      var c = d / 2.1 + e / 88 - t / 15;
      var X = q * Math.sin(c) + 200;
      var Y = (q + 6.5 * d) * Math.cos(c) + 8 * Math.sin(e * 0.5 + t) + 205;
      if (!finite(X, Y)) continue;
      outX[n] = X; outY[n] = Y; n++;
    }
    return n;
  }

  var CREATURES = [
    { id:"fucan", name:"北斗浮蚕", nameEn:"Beidou Fucan", tag:"环节浮游虫", tagEn:"pelagic annelid",
      blurb:"海报同款公式。透明小桨手，环节上会发黄光。",
      blurbEn:"The poster formula: a translucent paddler with a hint of yellow along its segments.",
      formulas: [
        "k = x/4 − 12.5 ,   e = y/9 + 6 ,   o = √(k²+e²)/9",
        "c = o/2 + e/2 − t/4",
        "q = (3/k)(½ tan(y/2)+cos y) + k(5/o + o·sin y·sin(e+4o−t))",
        "⟨ q + 40 cos(c) ,  q sin c − o k²/6 + 12 e o ⟩"
      ],
      fill: fillFucan, dt: Math.PI/90, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505",
      ink:"rgba(255,255,255,", inkA:0.50 },
    { id:"youyan", name:"蚰蜒", nameEn:"House Centipede", tag:"多足虫 · life 1", tagEn:"myriapod · life 1",
      blurb:"最经典的赛博节肢：tan(1/k) 画出关节，像百足在游。",
      blurbEn:"Classic cyber arthropod: tan(1/k) joints, swimming like a house centipede.",
      formulas: [
        "k = x/4 − 12.5 ,   e = y/9 + 5 ,   o = √(k²+e²)/9",
        "q = x+99 + tan(1/k) + o k (cos(9e)/4 + cos(y/2)) sin(4o−t)",
        "c = o e/30 − t/8",
        "⟨ 0.7 q sin c + 9 cos(y/19+t)+200 ,  200 + q/2 cos c ⟩"
      ],
      fill: fillYouyan, dt: Math.PI/90, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"jichong", name:"脊虫", nameEn:"Spine Worm", tag:"赛博脊虫 · life 2", tagEn:"cyber spine · life 2",
      blurb:"长条脊柱左右摆动，像深海里的一条脊梁。",
      blurbEn:"A long spine swaying side to side, like a backbone in the deep.",
      formulas: [
        "e = y/8 − 13",
        "k = (4 + 3 sin(2y−t)) cos(x/29) ,   d = √(k²+e²)",
        "q = 3 sin(2k) + 0.3/k + sin(y/25)·k·(9+4 sin(9e−3d+2t))",
        "⟨ q + 30 cos(d−t)+200 ,  620 − q sin(d−t) − 39d ⟩"
      ],
      fill: fillJichong, dt: Math.PI/240, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"jelly", name:"小水母", nameEn:"Jellyfish", tag:"伞状 · life 3", tagEn:"medusa · life 3",
      blurb:"Matlab 博客里讲过的那只。触须随 π/20 轻轻摇。",
      blurbEn:"The Matlab-blog jelly. Tentacles sway on a π/20 beat.",
      formulas: [
        "k = 5 cos(x/14) cos(y/30) ,   e = y/8 − 13",
        "d = (k²+e²)/59 + 4 ,   a = atan2(k,e)",
        "q = 60 − 3 sin(a e) + k (3 + 4/d sin(d²−2t))",
        "c = d/2 + e/99 − t/18",
        "⟨ q sin c + 200 ,  (q+9d) cos c + 200 ⟩"
      ],
      fill: fillJelly, dt: Math.PI/20, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"nebula", name:"星云水母", nameEn:"Nebula Jelly", tag:"大伞盖 · life 4", tagEn:"bell · life 4",
      blurb:"(k²+e²)/169 撑开伞盖，光斑像深海热泉。",
      blurbEn:"(k²+e²)/169 opens the bell; specks like a hydrothermal vent.",
      formulas: [
        "k = x/8 − 12.5 ,   e = y/8 − 12.5",
        "o = (k²+e²)/169 ,   d = ½ + 5 cos(o)",
        "X = x + d k sin(2d+o+t) + e cos(e+t) + 100",
        "Y = y/4 − 135 o + 6d cos(3d+9o+t) + 275"
      ],
      fill: fillNebula, dt: Math.PI/120, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"lantern", name:"花水母", nameEn:"Lantern Jelly", tag:"灯笼 · life 5", tagEn:"lantern · life 5",
      blurb:"atan2 绕七圈，伞缘开成一朵花。",
      blurbEn:"atan2 winds seven times; the bell rim opens like a flower.",
      formulas: [
        "k = 9 cos(x/8) ,   e = y/8 − 12.5",
        "d = (k²+e²)/99 + sin(t)/6 + ½",
        "q = 99 − e sin(7 atan2(k,e))/d + k(3+2 cos(d²−t))",
        "c = d/2 + e/69 − t/16",
        "⟨ q sin c + 200 ,  (q+19d) cos c + 200 ⟩"
      ],
      fill: fillLantern, dt: Math.PI/120, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"feather", name:"羽鳃", nameEn:"Feather Gill", tag:"life 6", tagEn:"life 6",
      blurb:"前半段用位运算扭出分节，后半段收成一条羽。",
      blurbEn:"Bitwise kinks in the first half, then it folds into a feather.",
      formulas: [
        "y = i/790 ,   k = y<5 ? 6+6 sin(⌊y⌋⊕1) : 4+cos y",
        "d = √( (k cos(i+t/4))² + (y/3−13)² )",
        "q = y k cos(i+t/4)/5 · (2+sin(2d+y−4t))",
        "c = d/3 − t/2 + (i mod 2)",
        "⟨ q+90 cos c+200 ,  400−(q sin c+29d−170) ⟩"
      ],
      fill: fillFeather, dt: Math.PI/90, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"tentacle", name:"触须虫", nameEn:"Tentacle Worm", tag:"life 7", tagEn:"life 7",
      blurb:"触手从一点伸出去再卷回来，像在探路。",
      blurbEn:"Arms reach out and coil back, as if feeling the way.",
      formulas: [
        "y = i/345 ,   x = y<11 ? 6+6 sin(⌊y⌋⊕8) : y/5+cos(y/2)",
        "e = y/7 − 13 ,   k = x cos(i−t/4)",
        "d = √(k²+e²) + ½ sin(e/4+t)",
        "q = y k/d · (3+sin(2d+y/2−4t)) ,   c = d/2+1−t/2",
        "⟨ q+60 cos c+200 ,  400−(q sin c+29d−170) ⟩"
      ],
      fill: fillTentacle, dt: Math.PI/120, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"flower6", name:"六瓣花", nameEn:"Six-petal", tag:"辐射对称 · life 8", tagEn:"radial · life 8",
      blurb:"同一组点转 6 次，开成海星或雪花。",
      blurbEn:"The same points rotated six times: a starfish or a snowflake.",
      formulas: [
        "k = (i mod 25)−12 ,   e = i/800",
        "d = 7 cos( √(k²+e²)/3 + t/2 )",
        "x₀ = 4k + d k sin(d+e/9+t)",
        "y₀ = 2e − 9d − 9d cos(d+t)",
        "旋转 π/3 × 6 ，再平移 +200"
      ],
      fill: fillFlower6, dt: Math.PI/240, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"wheel", name:"轮虫花", nameEn:"Rotifer Wheel", tag:"辐射对称 · life 9", tagEn:"radial · life 9",
      blurb:"转 14 瓣，像轮虫的纤毛圈。",
      blurbEn:"Fourteen petals, like a rotifer’s corona of cilia.",
      formulas: [
        "k = (i mod 50)−25 ,   e = i/1100",
        "d = 5 cos( √(k²+e²) − t + (i mod 2) )",
        "x₀ = k + (k d/6) sin(d+e/3+t)",
        "y₀ = 90 + e d − (2e/d) cos(d+t)",
        "旋转 π/7 × 14 ，再平移 +200"
      ],
      fill: fillWheel, dt: Math.PI/240, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"spiral", name:"螺灯", nameEn:"Spiral Lamp", tag:"原创 · 螺旋", tagEn:"original · spiral",
      blurb:"对数螺线裹一层呼吸膜，像深海里的一盏螺灯。",
      blurbEn:"A log spiral wrapped in a breathing film: a lamp in the dark.",
      formulas: [
        "k = x/5 − 12 ,   e = y/8 − 8 ,   o = √(k²+e²)/8",
        "c = 1.15 o + t/5",
        "q = 22 + 10 sin(0.8e+t) + k(1.6 + 0.35 sin(3o−t))",
        "⟨ q cos c + 10 sin(2e+t)+200 ,  0.88 q sin c + 200 ⟩"
      ],
      fill: fillSpiral, dt: Math.PI/90, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"comb", name:"栉水母", nameEn:"Comb Jelly", tag:"原创 · 梳板", tagEn:"original · ctenophore",
      blurb:"八列梳板用 atan2 划开，身体扁长，会轻轻侧摆。",
      blurbEn:"Eight comb rows from atan2; a flat body with a slow roll.",
      formulas: [
        "k = 7 cos(x/10) cos(y/35) ,   e = y/8 − 12",
        "d = (k²+e²)/70 + 3 ,   a = atan2(k,e)",
        "q = 48 − 4 sin(4a) + k(2.2 + 3/d sin(d²−t))",
        "c = d/2.4 + e/85 − t/14"
      ],
      fill: fillComb, dt: Math.PI/80, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"saweel", name:"锯鳗", nameEn:"Saw Eel", tag:"原创 · 长体", tagEn:"original · elongate",
      blurb:"比脊虫更密的锯齿，像一条带电的深海鳗。",
      blurbEn:"Denser serrations than the spine worm: an electric deep-sea eel.",
      formulas: [
        "e = y/9 − 12",
        "k = (3.5 + 2.4 sin(1.6y−t)) cos(x/22) ,   d = √(k²+e²)",
        "q = 2.2 sin(3k) + 0.25/k + sin(y/18)·k·(7+3 sin(6e−2d+2t))",
        "⟨ q + 24 cos(0.7d−t)+200 ,  560 − q sin(0.7d−t) − 32d ⟩"
      ],
      fill: fillSawEel, dt: Math.PI/180, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"star8", name:"八腕星", nameEn:"Octo Star", tag:"原创 · 辐射", tagEn:"original · radial",
      blurb:"同一组点转 8 次，开成八条腕的海星。",
      blurbEn:"The same points rotated eight times into an eight-armed star.",
      formulas: [
        "k = (i mod 20)−10 ,   e = i/900",
        "d = 6 cos( √(k²+e²)/4 + t/3 )",
        "x₀ = 5k + d k sin(d+t)",
        "y₀ = 2.5e − 8d cos(d+e/8+t) ，旋转 π/4 × 8"
      ],
      fill: fillStar8, dt: Math.PI/200, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"shrimp", name:"磷虾", nameEn:"Krill", tag:"原创 · 节肢", tagEn:"original · arthropod",
      blurb:"短体节肢，尾巴一甩一甩，像发光的磷虾。",
      blurbEn:"A short arthropod flicking its tail, like glowing krill.",
      formulas: [
        "k = x/4 − 12.5 ,   e = y/8 + 3.5 ,   o = √(k²+e²)/8",
        "q = 55 + 10 sin(0.8k) + k(2.2 + 0.55 o sin(0.7y−t))",
        "c = o/3.2 + e/22 − t/9",
        "⟨ 0.5 q sin c + 7 cos(y/16+t)+200 ,  200 + 0.38 q cos c ⟩"
      ],
      fill: fillShrimp, dt: Math.PI/70, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"vortex", name:"涡虫", nameEn:"Vortex Worm", tag:"原创 · 旋涡", tagEn:"original · vortex",
      blurb:"极坐标半径随三倍角起伏，整只虫子在原地打旋。",
      blurbEn:"Polar radius rises on a triple angle; the body spins in place.",
      formulas: [
        "x = (i mod 200)−100 ,   y = i/200 − 35",
        "r = √(x²+y²)/38 ,   θ = atan2(y,x)",
        "R = 62 + 22 sin(3θ+t) + 10 sin(5r−2t)",
        "⟨ R cos(θ+0.45r+t/7)+200 ,  0.9 R sin(θ+0.45r+t/7)+200 ⟩"
      ],
      fill: fillVortex, dt: Math.PI/100, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" },
    { id:"angel", name:"海天使", nameEn:"Sea Angel", tag:"原创 · 翼足", tagEn:"original · pteropod",
      blurb:"一对小翼拍水，身体像透明的海蝴蝶。",
      blurbEn:"A pair of tiny wings beating water: a transparent sea butterfly.",
      formulas: [
        "k = 6.5 cos(x/12) cos(y/38) ,   e = y/9 − 11",
        "d = (k²+e²)/68 + 2.4 ,   a = atan2(k,e)",
        "q = 38 − 5 sin(3a) + k(2 + 3.2/d sin(2.2d−t))",
        "c = d/2.1 + e/88 − t/15"
      ],
      fill: fillAngel, dt: Math.PI/90, view: {x0:0,x1:400,y0:0,y1:400}, bg:"#050505" }
  ];

  var idx = -1, t = 0, playing = true, lastTs = 0;
  var oceanInst = [], oceanSeed = 1, oceanT = 0, oceanFocus = -1, oceanHover = -1;
  var pointerX = 0.5, pointerY = 0.5, pointerDown = 0;
  var legendGeom = null, legendOff = null, legendOffCtx = null, legendScan = 0;
  var cv = document.getElementById("cv");
  var ctx = cv.getContext("2d", { alpha: false, desynchronized: true })
    || cv.getContext("2d", { alpha: false });
  var targetFps = (SAVER || WALLPAPER) ? 30 : 48;
  var frameMin = 1000 / targetFps;
  var fillBoost = 0;
  var oceanFrame = 0;
  var bgGrad = null, bgGradH = -1;
  var vigGrad = null, vigW = -1, vigH = -1;

  function mulberry32(a) {
    return function () {
      a |= 0; a = a + 0x6D2B79F5 | 0;
      var tt = Math.imul(a ^ a >>> 15, 1 | a);
      tt = tt + Math.imul(tt ^ tt >>> 7, 61 | tt) ^ tt;
      return ((tt ^ tt >>> 14) >>> 0) / 4294967296;
    };
  }
  function shuffled(n, rand) {
    var a = [], i, j, tmp;
    for (i = 0; i < n; i++) a.push(i);
    for (i = n - 1; i > 0; i--) {
      j = (rand() * (i + 1)) | 0;
      tmp = a[i]; a[i] = a[j]; a[j] = tmp;
    }
    return a;
  }

  function spawnOcean(seed) {
    seed = seed == null ? ((Math.random() * 0xffffffff) >>> 0) : (seed >>> 0);
    var rand = mulberry32(seed);
    var nC = CREATURES.length, cols = 5, rows = 4;
    var cells = shuffled(cols * rows, rand).slice(0, nC);
    var order = shuffled(nC, rand);
    var inst = [];
    for (var i = 0; i < nC; i++) {
      var cell = cells[i], col = cell % cols, row = (cell / cols) | 0;
      inst.push({
        ci: order[i],
        x: Math.min(0.90, Math.max(0.10, (col + 0.5 + (rand() - 0.5) * 0.32) / cols)),
        y: Math.min(0.84, Math.max(0.18, (row + 0.5 + (rand() - 0.5) * 0.32) / rows)),
        scale: 0.36 + rand() * 0.10,
        rot: (rand() - 0.5) * 0.2,
        t: 0.6 + rand() * 4,
        tempo: 1.15 + rand() * 0.55,
        vx: (rand() - 0.5) * 0.05,
        vy: (rand() - 0.5) * 0.032,
        wobble: 0.7 + rand() * 0.8,
        phase: rand() * Math.PI * 2,
        pulse: 0,
        fx: 0, fy: 0,
        _ax: 0, _ay: 0, _r: 0
      });
    }
    oceanSeed = seed;
    oceanInst = inst;
    oceanFocus = -1;
    oceanHover = -1;
  }

  function markCards() {
    var cards = document.querySelectorAll(".card");
    for (var k = 0; k < cards.length; k++) {
      var on = (k === 0 && idx < 0) || (k > 0 && idx === k - 1);
      var ocean = k === 0;
      cards[k].className = "card" + (ocean ? " ocean" : "") + (on ? " on" : "");
    }
  }

  function setFormula(lines) {
    var el = document.getElementById("formula");
    el.innerHTML = "";
    if (!lines || !lines.length) { el.hidden = true; return; }
    el.hidden = false;
    for (var i = 0; i < lines.length; i++) {
      var d = document.createElement("div");
      d.textContent = lines[i];
      el.appendChild(d);
    }
  }

  function select(i) {
    idx = i;
    t = 1.2;
    var C = CREATURES[idx];
    document.getElementById("title").textContent = "数字生命 · Digital Life";
    document.getElementById("sub").textContent = "集合馆 · Gallery";
    document.getElementById("kicker").textContent = C.tag + " · " + C.tagEn;
    document.getElementById("name").textContent = C.name + "  /  " + C.nameEn;
    document.getElementById("blurb").textContent = C.blurb;
    document.getElementById("blurbEn").textContent = C.blurbEn || "";
    document.getElementById("btnLesson").style.display = C.id === "fucan" ? "inline-block" : "none";
    document.getElementById("btnShuffle").style.display = "none";
    document.body.classList.remove("ocean");
    setFormula(C.formulas);
    markCards();
  }

  function selectOcean() {
    idx = -1;
    if (!oceanInst.length) spawnOcean(FIXED_SEED);
    document.getElementById("title").textContent = "数字生命 · Digital Life";
    document.getElementById("sub").textContent = "赛博海洋 · Cyber Ocean";
    document.getElementById("kicker").textContent = "随机群落 · Swarm";
    document.getElementById("name").textContent = "种子 " + oceanSeed + " · seed · " + oceanInst.length + " 只";
    document.getElementById("blurb").textContent = "白点公式生物会互相躲开，也会被指针推开。";
    document.getElementById("blurbEn").textContent = "Parametric creatures avoid each other and drift away from the pointer.";
    document.getElementById("btnLesson").style.display = "none";
    document.getElementById("btnShuffle").style.display = "inline-block";
    document.body.classList.add("ocean");
    setFormula([]);
    markCards();
  }

  var strip = document.getElementById("strip");
  var oceanCard = document.createElement("button");
  oceanCard.className = "card ocean on";
  oceanCard.type = "button";
  oceanCard.innerHTML = "<strong>赛博海洋</strong><span>Ocean · all</span>";
  oceanCard.onclick = function () { selectOcean(); };
  strip.appendChild(oceanCard);
  CREATURES.forEach(function (C, i) {
    var b = document.createElement("button");
    b.className = "card";
    b.type = "button";
    b.innerHTML = "<strong>" + C.name + "</strong><span>" + C.nameEn + "</span>";
    b.onclick = function () { select(i); };
    strip.appendChild(b);
  });
  selectOcean();

  document.getElementById("btnPause").onclick = function () {
    playing = !playing;
    this.textContent = playing ? "暂停 · Pause" : "继续 · Play";
  };
  document.getElementById("btnOcean").onclick = function () { selectOcean(); };
  document.getElementById("btnShuffle").onclick = function () {
    spawnOcean();
    document.getElementById("kicker").textContent = "随机群落 · Swarm";
    document.getElementById("name").textContent = "种子 " + oceanSeed + " · seed · " + oceanInst.length + " 只";
    document.getElementById("blurb").textContent = "白点公式生物会互相躲开，也会被指针推开。";
    document.getElementById("blurbEn").textContent = "Parametric creatures avoid each other and drift away from the pointer.";
  };

  window.addEventListener("mousemove", function (e) {
    pointerX = e.clientX / Math.max(1, window.innerWidth);
    pointerY = e.clientY / Math.max(1, window.innerHeight);
    if (!SAVER && idx < 0) {
      cv.classList.toggle("legend-hit", pickLegend(e) >= 0);
    }
  });
  window.addEventListener("mousedown", function () { pointerDown = 1; });
  window.addEventListener("mouseup", function () { pointerDown = 0; });
  window.addEventListener("keydown", function (e) {
    if (SAVER) return;
    if (idx < 0 && (e.key === "r" || e.key === "R")) {
      spawnOcean();
      document.getElementById("kicker").textContent = "随机群落 · Swarm";
      document.getElementById("name").textContent = "种子 " + oceanSeed + " · seed · " + oceanInst.length + " 只";
      document.getElementById("blurb").textContent = "白点公式生物会互相躲开，也会被指针推开。";
      document.getElementById("blurbEn").textContent = "Parametric creatures avoid each other and drift away from the pointer.";
    }
  });
  if (SAVER) {
    var saverArmed = false, saverX = 0, saverY = 0;
    setTimeout(function () { saverArmed = true; }, 1800);
    function quitSaver() {
      if (!saverArmed) return;
      saverArmed = false;
      try { fetch("/api/quit", { method: "POST", keepalive: true }); } catch (err) {}
      try { window.close(); } catch (err) {}
    }
    window.addEventListener("mousemove", function (e) {
      if (!saverArmed) { saverX = e.screenX; saverY = e.screenY; return; }
      if (Math.abs(e.screenX - saverX) + Math.abs(e.screenY - saverY) < 12) return;
      quitSaver();
    });
    window.addEventListener("mousedown", quitSaver);
    window.addEventListener("keydown", quitSaver);
    window.addEventListener("wheel", quitSaver, { passive: true });
    window.addEventListener("touchstart", quitSaver, { passive: true });
  }
  if (SAVER || WALLPAPER) {
    setInterval(function () { if (idx < 0) spawnOcean(); }, 180000);
  }
  function pickLegend(e) {
    if (idx >= 0 || !legendGeom || !oceanInst.length) return -1;
    var dpr = maxDpr();
    var x = e.clientX * dpr, y = e.clientY * dpr;
    var L = legendGeom;
    if (x < L.x0 || x > L.x0 + L.boxW || y < L.y0 || y > L.y0 + L.boxH) return -1;
    var k = ((y - L.y0 - L.head) / L.rowH) | 0;
    if (k < 0 || k >= oceanInst.length) return -1;
    return k;
  }
  function pickOcean(e, radiusMul) {
    if (idx >= 0 || !oceanInst.length) return -1;
    var dpr = maxDpr();
    var x = e.clientX * dpr, y = e.clientY * dpr;
    var best = -1, bestD = 1e12, k, inst, d, lim;
    for (k = 0; k < oceanInst.length; k++) {
      inst = oceanInst[k];
      d = (inst._ax - x) * (inst._ax - x) + (inst._ay - y) * (inst._ay - y);
      lim = Math.max(90 * dpr, inst._r * (radiusMul || 0.55));
      if (d < bestD && d < lim * lim) { bestD = d; best = k; }
    }
    return best;
  }
  cv.addEventListener("click", function (e) {
    var best = pickLegend(e);
    if (best < 0) best = pickOcean(e, 0.7);
    if (best < 0) return;
    oceanFocus = best;
    oceanInst[best].pulse = 1;
    oceanInst[best].t += 0.8;
    var j, inst, dx, dy, dist;
    for (j = 0; j < oceanInst.length; j++) {
      if (j === best) continue;
      inst = oceanInst[j];
      dx = inst.x - oceanInst[best].x;
      dy = inst.y - oceanInst[best].y;
      dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
      if (dist < 0.28) {
        inst.vx += (dx / dist) * 0.08;
        inst.vy += (dy / dist) * 0.06;
        inst.pulse = Math.max(inst.pulse, 0.35);
      }
    }
  });

  function layoutLegend(w, h, dpr) {
    var n = Math.max(1, oceanInst.length);
    var x0 = 14 * dpr;
    var y0 = (SAVER || WALLPAPER) ? 16 * dpr : 58 * dpr;
    var bot = (SAVER || WALLPAPER) ? 16 * dpr : 108 * dpr;
    var maxH = Math.max(90 * dpr, h - y0 - bot);
    var head = 34 * dpr;
    var rowH = Math.min(24 * dpr, Math.max(13.5 * dpr, (maxH - head - 10 * dpr) / n));
    var boxH = head + n * rowH + 10 * dpr;
    var boxW = Math.min(280 * dpr, Math.max(196 * dpr, w * 0.26));
    return { x0: x0, y0: y0, boxW: boxW, boxH: boxH, rowH: rowH, head: head, n: n };
  }

  function ensureLegendOff(bw, bh, clear) {
    if (!legendOff) legendOff = document.createElement("canvas");
    if (legendOff.width !== (bw | 0) || legendOff.height !== (bh | 0)) {
      legendOff.width = bw | 0;
      legendOff.height = bh | 0;
      clear = true;
    }
    legendOffCtx = legendOff.getContext("2d");
    if (clear) {
      legendOffCtx.setTransform(1, 0, 0, 1, 0, 0);
      legendOffCtx.clearRect(0, 0, legendOff.width, legendOff.height);
    }
  }

  function legendHighlight() {
    var n = oceanInst.length;
    if (!n) return -1;
    if (oceanFocus >= 0) return oceanFocus;
    if (oceanHover >= 0) return oceanHover;
    if (SAVER || WALLPAPER) return (legendScan | 0) % n;
    return -1;
  }

  function drawLegendHud(w, h, dpr, mx, my) {
    var L = legendGeom;
    if (!L || !oceanInst.length) return;
    var hi = legendHighlight();
    var x0 = L.x0, y0 = L.y0, bw = L.boxW, bh = L.boxH;
    var r = 14 * dpr;

    ctx.save();
    ctx.fillStyle = "rgba(3, 10, 14, 0.72)";
    ctx.strokeStyle = "rgba(90, 255, 220, 0.28)";
    ctx.lineWidth = Math.max(1, dpr);
    ctx.beginPath();
    if (ctx.roundRect) ctx.roundRect(x0, y0, bw, bh, r);
    else ctx.rect(x0, y0, bw, bh);
    ctx.fill();
    ctx.stroke();

    var scanY = y0 + L.head + ((legendScan % Math.max(1, oceanInst.length)) * L.rowH);
    var bar = ctx.createLinearGradient(x0, scanY - 8 * dpr, x0, scanY + L.rowH + 8 * dpr);
    bar.addColorStop(0, "rgba(80,255,220,0)");
    bar.addColorStop(0.5, "rgba(80,255,220,0.10)");
    bar.addColorStop(1, "rgba(80,255,220,0)");
    ctx.fillStyle = bar;
    ctx.fillRect(x0 + 2 * dpr, scanY - 6 * dpr, bw - 4 * dpr, L.rowH + 12 * dpr);

    ctx.fillStyle = "rgba(127,255,212,0.92)";
    ctx.font = (11 * dpr) + "px 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif";
    ctx.textAlign = "left";
    ctx.textBaseline = "alphabetic";
    ctx.fillText("图例  /  Legend", x0 + 14 * dpr, y0 + 16 * dpr);
    ctx.fillStyle = "rgba(180,230,220,0.55)";
    ctx.font = (9 * dpr) + "px 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif";
    ctx.fillText("种子 seed " + oceanSeed + " · " + oceanInst.length, x0 + 14 * dpr, y0 + 28 * dpr);

    if (legendOff) ctx.drawImage(legendOff, x0, y0);

    var k, inst, C, ly, on, depth, labelX;
    ctx.textBaseline = "middle";
    for (k = 0; k < oceanInst.length; k++) {
      inst = oceanInst[k];
      C = CREATURES[inst.ci];
      ly = y0 + L.head + (k + 0.5) * L.rowH;
      on = k === hi;
      depth = 0.45 + (1 - inst.y) * 0.55;
      if (on) {
        ctx.fillStyle = "rgba(80,255,230,0.12)";
        ctx.fillRect(x0 + 6 * dpr, y0 + L.head + k * L.rowH, bw - 12 * dpr, L.rowH);
      }
      labelX = x0 + 42 * dpr;
      ctx.fillStyle = on ? "rgba(255,255,255,0.95)" : "rgba(210,245,235," + (0.62 * depth) + ")";
      ctx.font = (on ? "600 " : "500 ") + (11 * dpr) + "px 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif";
      ctx.fillText(C.name, labelX, ly - 3.2 * dpr);
      ctx.fillStyle = on ? "rgba(127,255,212,0.8)" : "rgba(140,190,180," + (0.45 * depth) + ")";
      ctx.font = (8.5 * dpr) + "px 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif";
      ctx.fillText(C.nameEn || C.tag, labelX, ly + 8.2 * dpr);
    }

    if (hi >= 0 && oceanInst[hi] && oceanInst[hi]._ax) {
      inst = oceanInst[hi];
      ly = y0 + L.head + (hi + 0.5) * L.rowH;
      ctx.strokeStyle = "rgba(160,255,240," + (0.28 + 0.2 * (inst.pulse || 0)) + ")";
      ctx.lineWidth = Math.max(1, dpr * 0.9);
      ctx.setLineDash([5 * dpr, 5 * dpr]);
      ctx.lineDashOffset = -oceanT * 28 * dpr;
      ctx.beginPath();
      ctx.moveTo(x0 + bw - 8 * dpr, ly);
      ctx.bezierCurveTo(
        x0 + bw + 48 * dpr, ly,
        (x0 + bw + inst._ax) / 2, (ly + inst._ay) / 2,
        inst._ax, inst._ay
      );
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = "rgba(180,255,240,0.85)";
      ctx.beginPath();
      ctx.arc(x0 + bw - 8 * dpr, ly, 2.2 * dpr, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  function drawOceanBg(w, h, now, dpr) {
    if (!bgGrad || bgGradH !== h) {
      bgGradH = h;
      bgGrad = ctx.createLinearGradient(0, 0, 0, h);
      bgGrad.addColorStop(0, "#04140c");
      bgGrad.addColorStop(0.22, "#031018");
      bgGrad.addColorStop(0.55, "#02101c");
      bgGrad.addColorStop(1, "#01060a");
    }
    ctx.fillStyle = bgGrad;
    ctx.fillRect(0, 0, w, h);

    var horizon = h * 0.36;
    var vpX = w * (0.5 + (pointerX - 0.5) * 0.08);
    var i, x, y;
    var saver = SAVER || WALLPAPER;
    var nRay = saver ? 2 : 5;
    var nGrid = saver ? 8 : 16;

    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    for (i = 0; i < nRay; i++) {
      x = w * (0.18 + i * (saver ? 0.36 : 0.18)) + Math.sin(now * 0.11 + i * 1.7) * w * 0.05;
      var ray = ctx.createLinearGradient(x, 0, x - w * 0.08, h);
      ray.addColorStop(0, saver ? "rgba(90,230,255,0.07)" : "rgba(90,230,255,0.10)");
      ray.addColorStop(0.55, "rgba(40,180,160,0.03)");
      ray.addColorStop(1, "rgba(0,0,0,0)");
      ctx.fillStyle = ray;
      ctx.beginPath();
      ctx.moveTo(x - 18 * dpr, 0);
      ctx.lineTo(x + 52 * dpr, 0);
      ctx.lineTo(x - 70 * dpr, h);
      ctx.lineTo(x - 210 * dpr, h);
      ctx.closePath();
      ctx.fill();
    }
    ctx.restore();

    ctx.save();
    ctx.beginPath();
    ctx.rect(0, horizon, w, h - horizon);
    ctx.clip();
    ctx.strokeStyle = "rgba(0,255,200,0.07)";
    ctx.lineWidth = Math.max(1, dpr * 0.7);
    for (i = 1; i <= nGrid; i++) {
      y = horizon + Math.pow(i / nGrid, 1.55) * (h - horizon);
      ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
    }
    for (i = -nGrid; i <= nGrid; i++) {
      ctx.beginPath();
      ctx.moveTo(vpX + i * w * 0.065, horizon);
      ctx.lineTo(vpX + i * w * 0.5, h);
      ctx.stroke();
    }
    ctx.restore();

    ctx.fillStyle = "rgba(0,220,210,0.08)";
    ctx.fillRect(0, horizon - 2 * dpr, w, 3 * dpr);

    ctx.strokeStyle = "rgba(80,255,220,0.08)";
    ctx.lineWidth = Math.max(1, dpr);
    var nWave = saver ? 4 : 7;
    var waveStep = saver ? 22 : 12;
    for (i = 0; i < nWave; i++) {
      ctx.beginPath();
      var amp = 8 * dpr + i * 2.2;
      var y0 = h * (0.10 + i * (saver ? 0.16 : 0.10));
      ctx.moveTo(0, y0);
      for (x = 0; x <= w; x += waveStep) {
        ctx.lineTo(x, y0 + Math.sin(x * 0.0065 + now * (0.32 + i * 0.07) + i) * amp
          + Math.sin(x * 0.018 - now * 0.26) * amp * 0.35);
      }
      ctx.stroke();
    }

    ctx.fillStyle = "rgba(180,255,230,0.16)";
    var nSpark = saver ? 28 : 55;
    for (i = 0; i < nSpark; i++) {
      var u = (i * 0.618034 + now * 0.016) % 1;
      var v = (i * 0.371 + 0.07) % 1;
      ctx.fillRect(
        (u + Math.sin(now * 0.12 + i) * 0.02) * w,
        v * h,
        1 + (i % 3), 1 + (i % 2)
      );
    }

    ctx.strokeStyle = "rgba(160,255,255,0.12)";
    ctx.lineWidth = Math.max(1, dpr * 0.8);
    var nBub = saver ? 8 : 16;
    for (i = 0; i < nBub; i++) {
      var bu = (i * 0.173 + now * (0.04 + (i % 5) * 0.008)) % 1;
      x = ((i * 97) % 1000) / 1000 * w;
      y = h - bu * h * 1.15;
      var br = (2 + (i % 5)) * dpr;
      ctx.beginPath();
      ctx.arc(x, y, br, 0, Math.PI * 2);
      ctx.stroke();
    }

    if (vigW !== w || vigH !== h || !vigGrad) {
      vigW = w; vigH = h;
      vigGrad = ctx.createRadialGradient(w * 0.5, h * 0.42, h * 0.12, w * 0.5, h * 0.5, h * 0.78);
      vigGrad.addColorStop(0, "rgba(0,0,0,0)");
      vigGrad.addColorStop(1, "rgba(0,0,0,0.42)");
    }
    ctx.fillStyle = vigGrad;
    ctx.fillRect(0, 0, w, h);
  }

  function drawOcean(w, h, dpr, dt) {
    if (playing) oceanT += dt;
    drawOceanBg(w, h, oceanT, dpr);

    var topG = (SAVER || WALLPAPER) ? 12 * dpr : 64 * dpr;
    var botG = (SAVER || WALLPAPER) ? 12 * dpr : 92 * dpr;
    var sideG = 8 * dpr;
    var innerW = Math.max(40, w - 2 * sideG);
    var innerH = Math.max(40, h - topG - botG);
    var world = Math.min(innerW, innerH);
    var camX = (pointerX - 0.5) * 40 * dpr;
    var camY = (pointerY - 0.5) * 22 * dpr;
    var current = Math.sin(oceanT * 0.22) * 0.016;
    var mx = pointerX * w, my = pointerY * h;
    var i, k, inst, C, v, n, cx, cy, ca, sa, sc, px, py, X, Y, dx, dy, ax, ay, s;
    var driftX, driftY, rdx, rdy, rd, rd2, force, breathe, sway, near, hover = -1, hoverD = 1e12;
    var L, hi, lx, ly, msc, ms;

    legendGeom = layoutLegend(w, h, dpr);
    L = legendGeom;
    if (playing) legendScan = (legendScan + dt * 0.42) % Math.max(1, oceanInst.length);
    oceanFrame++;
    var redrawLegend = (oceanFrame % 3) === 0;
    ensureLegendOff(L.boxW, L.boxH, redrawLegend);
    hi = legendHighlight();

    FILL_STEP = ((SAVER || WALLPAPER) ? 4 : (oceanInst.length > 12 ? 3 : 2)) + fillBoost;
    ctx.fillStyle = "rgba(255,255,255,0.30)";
    s = Math.max(1.3, Math.min(w, h) / 560);

    if (playing) {
      var a, b, oa, ob, ddx, ddy, dd, sep, push, nx, ny;
      for (a = 0; a < oceanInst.length; a++) {
        oceanInst[a].fx = 0;
        oceanInst[a].fy = 0;
      }
      for (a = 0; a < oceanInst.length; a++) {
        oa = oceanInst[a];
        for (b = a + 1; b < oceanInst.length; b++) {
          ob = oceanInst[b];
          ddx = oa.x - ob.x;
          ddy = oa.y - ob.y;
          dd = Math.sqrt(ddx * ddx + ddy * ddy);
          sep = (oa.scale + ob.scale) * 0.34;
          if (dd < 1e-4) {
            ddx = Math.cos(oa.phase + oceanT);
            ddy = Math.sin(oa.phase + oceanT);
            dd = 1;
          }
          if (dd < sep) {
            nx = ddx / dd;
            ny = ddy / dd;
            push = (1 - dd / sep);
            push *= push;
            if (dd < sep * 0.55) push *= 2.1;
            oa.fx += nx * push;
            oa.fy += ny * push;
            ob.fx -= nx * push;
            ob.fy -= ny * push;
          }
        }
      }
    }

    for (k = 0; k < oceanInst.length; k++) {
      inst = oceanInst[k];
      C = CREATURES[inst.ci];
      if (playing) {
        inst.t += C.dt * dt * 60 * inst.tempo;
        if (inst.pulse > 0) {
          inst.t += C.dt * dt * 80 * inst.pulse;
          inst.pulse = Math.max(0, inst.pulse - dt * 1.6);
        }
        inst.vx += inst.fx * dt * 1.4;
        inst.vy += inst.fy * dt * 1.4;
        var sp = Math.sqrt(inst.vx * inst.vx + inst.vy * inst.vy);
        if (sp > 0.09) { inst.vx *= 0.09 / sp; inst.vy *= 0.09 / sp; }
        driftX = inst.vx + Math.sin(oceanT * inst.wobble + inst.phase) * 0.032 + current + inst.fx * 0.85;
        driftY = inst.vy + Math.cos(oceanT * inst.wobble * 0.73 + inst.phase) * 0.022 + inst.fy * 0.85;
        inst.x += driftX * dt;
        inst.y += driftY * dt;
        px = sideG + inst.x * innerW + camX;
        py = topG + inst.y * innerH + camY;
        rdx = px - mx; rdy = py - my;
        rd2 = rdx * rdx + rdy * rdy;
        var rMin = 210 * dpr;
        if (rd2 < rMin * rMin && rd2 > 16) {
          rd = Math.sqrt(rd2);
          force = (1 - rd / rMin) * (pointerDown ? 0.55 : 0.28);
          inst.x += (rdx / rd) * force * dt * 1.8;
          inst.y += (rdy / rd) * force * dt * 1.8;
        }
        if (inst.x < 0.10) { inst.x = 0.10; inst.vx = Math.abs(inst.vx); }
        if (inst.x > 0.90) { inst.x = 0.90; inst.vx = -Math.abs(inst.vx); }
        if (inst.y < 0.18) { inst.y = 0.18; inst.vy = Math.abs(inst.vy); }
        if (inst.y > 0.84) { inst.y = 0.84; inst.vy = -Math.abs(inst.vy); }
      }
      n = C.fill(inst.t);
      v = C.view;
      cx = (v.x0 + v.x1) / 2;
      cy = (v.y0 + v.y1) / 2;
      sway = 0.16 * Math.sin(oceanT * inst.wobble + inst.phase);
      ca = Math.cos(inst.rot + sway);
      sa = Math.sin(inst.rot + sway);
      breathe = 1 + 0.07 * Math.sin(oceanT * 1.6 + inst.phase) + inst.pulse * 0.22;
      if (k === oceanFocus || k === oceanHover) breathe += 0.10;
      sc = inst.scale * breathe * world / (v.x1 - v.x0);
      px = sideG + inst.x * innerW + camX;
      py = topG + inst.y * innerH + camY;
      inst._ax = px; inst._ay = py; inst._r = sc * 155;
      near = (px - mx) * (px - mx) + (py - my) * (py - my);
      if (near < hoverD) { hoverD = near; hover = k; }
      if (k === oceanFocus || k === oceanHover) ctx.fillStyle = "rgba(255,255,255,0.42)";
      for (i = 0; i < n; i++) {
        X = outX[i]; Y = outY[i];
        if (X < v.x0 || X > v.x1 || Y < v.y0 || Y > v.y1) continue;
        dx = X - cx; dy = Y - cy;
        ax = px + (dx * ca - dy * sa) * sc;
        ay = py - (dx * sa + dy * ca) * sc;
        ctx.fillRect(ax, ay, s, s);
      }
      if (legendOffCtx && redrawLegend) {
        lx = 22 * dpr;
        ly = L.head + (k + 0.5) * L.rowH;
        msc = (L.rowH * 0.82) / (v.x1 - v.x0);
        ms = Math.max(0.8, dpr * 0.7);
        legendOffCtx.save();
        legendOffCtx.beginPath();
        legendOffCtx.rect(4 * dpr, L.head + k * L.rowH, 36 * dpr, L.rowH);
        legendOffCtx.clip();
        legendOffCtx.fillStyle = (k === hi)
          ? "rgba(255,255,255,0.92)"
          : "rgba(170,255,230,0.55)";
        for (i = 0; i < n; i += 4) {
          X = outX[i]; Y = outY[i];
          if (X < v.x0 || X > v.x1 || Y < v.y0 || Y > v.y1) continue;
          dx = X - cx; dy = Y - cy;
          legendOffCtx.fillRect(
            lx + (dx * ca - dy * sa) * msc,
            ly - (dx * sa + dy * ca) * msc,
            ms, ms
          );
        }
        legendOffCtx.restore();
      }
      if (k === oceanFocus || k === oceanHover) ctx.fillStyle = "rgba(255,255,255,0.30)";
    }
    FILL_STEP = 1;

    var hoverLim = hover >= 0 ? Math.max(70 * dpr, oceanInst[hover]._r * 0.62) : 0;
    if (hover >= 0 && hoverD < hoverLim * hoverLim) {
      oceanHover = hover;
    } else {
      oceanHover = -1;
    }
    if (L && mx >= L.x0 && mx <= L.x0 + L.boxW && my >= L.y0 + L.head && my <= L.y0 + L.boxH) {
      k = ((my - L.y0 - L.head) / L.rowH) | 0;
      if (k >= 0 && k < oceanInst.length) oceanHover = k;
    }

    if (!SAVER) {
      ctx.save();
      ctx.strokeStyle = "rgba(180,255,255," + (pointerDown ? 0.28 : 0.14) + ")";
      ctx.lineWidth = dpr;
      ctx.beginPath();
      ctx.arc(mx, my, (pointerDown ? 46 : 28) * dpr, 0, Math.PI * 2);
      ctx.stroke();
      ctx.restore();
    }

    var ring = oceanFocus >= 0 ? oceanFocus : oceanHover;
    if (ring >= 0 && oceanInst[ring]) {
      inst = oceanInst[ring];
      ctx.strokeStyle = "rgba(255,255,255,0.5)";
      ctx.lineWidth = 1.2 * dpr;
      ctx.beginPath();
      ctx.arc(inst._ax, inst._ay, Math.max(34 * dpr, inst._r * 0.34), 0, Math.PI * 2);
      ctx.stroke();
    }
    drawLegendHud(w, h, dpr, mx, my);
  }

  function maxDpr() {
    var raw = window.devicePixelRatio || 1;
    if (SAVER || WALLPAPER) return Math.min(raw, 1);
    return Math.min(raw, 1.5);
  }
  function schedule() {
    if (SAVER || WALLPAPER) {
      var wait = Math.max(0, frameMin - (performance.now() - lastTs));
      setTimeout(function () { requestAnimationFrame(draw); }, wait);
    } else {
      requestAnimationFrame(draw);
    }
  }
  function draw(ts) {
    if (!lastTs) lastTs = ts;
    var gap = ts - lastTs;
    if (gap < frameMin * 0.72) {
      schedule();
      return;
    }
    var dt = Math.min(0.05, gap / 1000);
    lastTs = ts;
    if (dt > 0.04) fillBoost = Math.min(3, fillBoost + 1);
    else if (dt < 0.024 && fillBoost > 0) fillBoost -= 1;

    var dpr = maxDpr();
    var wCss = window.innerWidth;
    var hCss = window.innerHeight;
    if (cv.width !== (wCss * dpr | 0) || cv.height !== (hCss * dpr | 0)) {
      cv.width = wCss * dpr | 0; cv.height = hCss * dpr | 0;
      bgGrad = null; vigGrad = null;
    }
    var w = cv.width, h = cv.height;
    ctx.setTransform(1,0,0,1,0,0);

    if (idx < 0) {
      drawOcean(w, h, dpr, dt);
      schedule();
      return;
    }

    FILL_STEP = 1 + (fillBoost > 1 ? 1 : 0);
    var C = CREATURES[idx];
    if (playing) t += C.dt * dt * 60;
    count = C.fill(t);

    ctx.fillStyle = C.bg;
    ctx.fillRect(0, 0, w, h);

    var v = C.view;
    var dx = v.x1 - v.x0, dy = v.y1 - v.y0;
    var topG = 72 * dpr, botG = 96 * dpr, sideG = 16 * dpr;
    var innerW = Math.max(40, w - 2 * sideG);
    var innerH = Math.max(40, h - topG - botG);
    var scale = Math.min(innerW / dx, innerH / dy);
    var ox = (w - dx * scale) / 2;
    var oy = topG + (innerH - dy * scale) / 2;
    var s = Math.max(1.4, Math.min(w, h) / 520);
    ctx.fillStyle = C.ink ? (C.ink + (C.inkA || 0.28) + ")") : "rgba(255,255,255,0.28)";
    for (var i = 0; i < count; i++) {
      var X = outX[i], Y = outY[i];
      if (X < v.x0 || X > v.x1 || Y < v.y0 || Y > v.y1) continue;
      var ax = ox + (X - v.x0) * scale;
      var ay = oy + (v.y1 - Y) * scale;
      ctx.fillRect(ax, ay, s, s);
    }
    FILL_STEP = 1;

    schedule();
  }
  if (SHOT) {
    document.body.classList.add("ocean");
    idx = -1;
    if (!oceanInst.length) spawnOcean(FIXED_SEED == null ? 42 : FIXED_SEED);
    var dpr0 = Math.min(window.devicePixelRatio || 1, 2);
    cv.width = (window.innerWidth * dpr0) | 0;
    cv.height = (window.innerHeight * dpr0) | 0;
    playing = true;
    var wi;
    for (wi = 0; wi < 160; wi++) drawOcean(cv.width, cv.height, dpr0, 1 / 40);
    return;
  }
  requestAnimationFrame(draw);
})();
  </script>
</body>
</html>
"""
