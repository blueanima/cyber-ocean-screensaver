"""北斗浮蚕海报 + 细致分步拆解（每步都有可见变化与过渡动画）。"""

INDEX_HTML = r"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" />
  <title>北斗浮蚕 · 细致分步</title>
  <style>
    :root {
      --bg: #050505; --fg: #ffffff; --accent: #ffe08a;
      --glass: rgba(6, 10, 12, 0.62); --line: rgba(255,255,255,0.14);
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    html, body {
      height: 100%; width: 100%; overflow: hidden; background: #050505; color: var(--fg);
      font-family: "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
    }
    canvas { position: fixed; inset: 0; width: 100%; height: 100%; display: block; background: var(--bg); }
    .topbar {
      position: fixed; top: 0; left: 0; right: 0; z-index: 3;
      display: flex; align-items: center; gap: 0.75rem;
      padding: 0.6rem 1rem 0.7rem;
      background: linear-gradient(180deg, rgba(0,0,0,0.78), rgba(0,0,0,0.18) 80%, transparent);
    }
    a.back {
      color: #e8ffe8; background: rgba(255,255,255,0.1);
      border: 1px solid rgba(255,255,255,0.22); text-decoration: none;
      border-radius: 999px; padding: 0.38rem 0.8rem; font-size: 0.8rem; font-weight: 600;
      flex: 0 0 auto;
    }
    .brand { min-width: 0; }
    .brand h1 { font-size: 0.92rem; font-weight: 650; letter-spacing: 0.16em; color: var(--accent); }
    .brand p { margin-top: 0.12rem; font-size: 0.72rem; color: rgba(200,230,200,0.8); }
    .panel-top {
      position: fixed; top: 3.9rem; left: 50%; transform: translateX(-50%);
      z-index: 3; width: min(720px, calc(100vw - 2rem));
      padding: 0.7rem 1rem 0.8rem; text-align: center;
      background: var(--glass); border: 1px solid var(--line); border-radius: 16px;
      backdrop-filter: blur(14px); -webkit-backdrop-filter: blur(14px);
      pointer-events: none;
    }
    .panel-top h2 { font-size: 1.05rem; font-weight: 650; color: var(--accent); line-height: 1.35; }
    .panel-top .formula {
      margin-top: 0.4rem; font-family: Georgia, "Times New Roman", serif;
      font-size: 0.84rem; color: rgba(255,250,210,0.92); line-height: 1.5;
    }
    .dock {
      position: fixed; left: 0; right: 0; bottom: 0; z-index: 3;
      padding: 0.7rem 1rem 0.85rem;
      background: linear-gradient(0deg, rgba(0,0,0,0.88), rgba(0,0,0,0.42) 70%, transparent);
    }
    .dock .explain {
      text-align: center; font-size: 0.84rem; color: #d8f3d8; line-height: 1.5;
      max-width: 720px; margin: 0 auto;
    }
    .dock .axis {
      text-align: center; margin-top: 0.25rem;
      font-size: 0.75rem; color: var(--accent);
    }
    .bar {
      height: 4px; margin: 0.55rem auto 0.55rem; max-width: 420px; border-radius: 99px;
      background: rgba(255,255,255,0.12); overflow: hidden;
    }
    .bar > i { display: block; height: 100%; background: var(--accent); width: 0; }
    .controls { display: flex; gap: 0.4rem; justify-content: center; flex-wrap: wrap; }
    button {
      border: 0; border-radius: 999px; padding: 0.42rem 0.85rem;
      font-size: 0.84rem; font-weight: 600; cursor: pointer;
      color: #163018; background: linear-gradient(180deg, #ffe9a8, #e7c04a);
    }
    button.ghost {
      color: #e8ffe8; background: rgba(255,255,255,0.1);
      border: 1px solid rgba(255,255,255,0.22);
    }
    button:disabled { opacity: 0.4; cursor: default; }
  </style>
</head>
<body>
  <canvas id="cv"></canvas>
  <header class="topbar">
    <a class="back" href="/">集合馆</a>
    <div class="brand">
      <h1>北斗浮蚕</h1>
      <p id="meta">分步讲解</p>
    </div>
  </header>
  <section class="panel-top" id="lessonTop">
    <h2 id="ltitle"></h2>
    <p class="formula" id="lformula"></p>
  </section>
  <div class="dock" id="lessonBot">
    <p class="explain" id="lexplain"></p>
    <p class="axis" id="laxis"></p>
    <div class="bar"><i id="bar"></i></div>
    <div class="controls">
      <button class="ghost" id="btnPrev" type="button">上一步</button>
      <button id="btnNext" type="button">下一步</button>
      <button class="ghost" id="btnAuto" type="button">自动讲解</button>
      <button class="ghost" id="btnPlay" type="button">播放游动</button>
    </div>
  </div>
  <script>
(function () {
  "use strict";

  var N = 20000;
  var xs = new Float32Array(N), ys = new Float32Array(N);
  for (var i = 0; i < N; i++) { xs[i] = i % 100; ys[i] = (i / 100) | 0; }

  var fromX = new Float32Array(N), fromY = new Float32Array(N);
  var toX = new Float32Array(N), toY = new Float32Array(N);
  var px = new Float32Array(N), py = new Float32Array(N);
  var aliveFrom = new Uint8Array(N), aliveTo = new Uint8Array(N);

  var STEPS = [
    { ch:"采样", title:"先只有横坐标 x",
      formula:"x = i mod 100",
      explain:"i 从 0、1、2… 往上数，先只变成 x。点排成一条横线。下一步会突然“长出”上下。",
      plot:"x_only", axis:"横：x　　竖：还没有 y" },
    { ch:"采样", title:"再加入 y，铺成格子",
      formula:"y = ⌊i / 100⌋",
      explain:"每 100 个点换一行。看点从一条线散开成点阵——这是全部原料。",
      plot:"xy", axis:"横：x　　竖：y" },
    { ch:"中间量", title:"k 把左右居中、缩小",
      formula:"k = x/4 − 12.5",
      explain:"格子在横向上往中间收。中心变成 0：左边负、右边正。",
      plot:"k_y", axis:"横：k　　竖：y（还没压矮）" },
    { ch:"中间量", title:"e 把上下压矮",
      formula:"e = y/9 + 6",
      explain:"竖向被压矮并往上托。注意：这一步横轴先回到 x，好对比“只改上下”。",
      plot:"x_e", axis:"横：x（未改）　　竖：e" },
    { ch:"中间量", title:"(k, e) 合在一起",
      formula:"横 = k，　竖 = e",
      explain:"左右用 k、上下用 e。后面所有公式都在这个新坐标系里算。",
      plot:"ke", axis:"横：k　　竖：e" },
    { ch:"中间量", title:"o 是到中心的距离",
      formula:"o = √(k² + e²) / 9",
      explain:"竖轴改成距离 o。越靠边越高，中间最低，像一座“山谷”。",
      plot:"k_o", axis:"横：k　　竖：o（距离）" },
    { ch:"角度 c", title:"c 的第一块：o/2",
      formula:"先写下 o/2",
      explain:"朝向先只用距离的一半。点重新按 o/2 横着排开。",
      plot:"o2_e", axis:"横：o/2　　竖：e" },
    { ch:"角度 c", title:"再加上 e/2（看点斜过去）",
      formula:"c ← o/2 + e/2",
      explain:"每个点按自己的高度再偏一点，整幅图会斜切变形。这不是平移，所以形状会变。",
      plot:"c_no_t", axis:"横：o/2 + e/2　　竖：e" },
    { ch:"角度 c", title:"减去 t/4 —— 整幅一起滑动",
      formula:"c = o/2 + e/2 − t/4",
      explain:"t/4 对每个点都一样，所以是整体平移。镜头故意锁住，你才能看见它在滑。t 会自己慢慢增加。",
      plot:"c_e", axis:"横：c　　竖：e", loopT:true, lockView:true },
    { ch:"幅度 q", title:"起伏 u = ½tan(y/2)+cos y",
      formula:"u = ½ tan(y/2) + cos y",
      explain:"tan 会在某些 y 炸掉，那些点被丢掉，于是出现一段段空隙——这就是以后的“环节缝”。",
      plot:"u_e", axis:"横：u　　竖：e" },
    { ch:"幅度 q", title:"乘上 3/k，中线附近被放大",
      formula:"q₁ = (3/k) · u",
      explain:"越靠近 k=0 放大越狠。左右两侧被拉开，q 的第一段成形。",
      plot:"q1_e", axis:"横：q₁　　竖：e" },
    { ch:"幅度 q", title:"q₂ 带着时间抖动",
      formula:"q₂ = k(5/o + o·sin y·sin(e+4o−t))",
      explain:"这里有 sin(…−t)。t 在动，点会来回扭——浮蚕“活着”很多靠这一段。",
      plot:"q2_e", axis:"横：q₂　　竖：e", loopT:true },
    { ch:"幅度 q", title:"q = q₁ + q₂，仍只是数字",
      formula:"q = q₁ + q₂",
      explain:"两段加起来。横轴是 q，还不是画面坐标。看点从 q₂ 的形状融进更宽的 q。",
      plot:"q_e", axis:"横：q（远近数字）　　竖：e" },
    { ch:"拼坐标", title:"用 c 转弯：像圆规",
      formula:"横 = 40 cos(c)，　竖 = q sin(c)",
      explain:"q 当长度、c 当角度。点会从“数字条”弯成一条虫的雏形。",
      plot:"rot", axis:"横：40 cos(c)　　竖：q sin(c)" },
    { ch:"拼坐标", title:"横的再加上 q，身体撑开",
      formula:"X = q + 40 cos(c)",
      explain:"在转弯之外把 q 加到横坐标。左右被撑开，环节变清楚。",
      plot:"x_full_yrot", axis:"横：q + 40 cos(c)　　竖：q sin(c)" },
    { ch:"拼坐标", title:"竖的加上 12eo，身体拉长",
      formula:"Y ← q sin(c) + 12 e o",
      explain:"12eo 把环节从上排到下。看点向上“长高”，不再挤成一团。",
      plot:"y_spine", axis:"横：X　　竖：q sin(c) + 12eo" },
    { ch:"拼坐标", title:"减去 ok²/6，腰收细，成形",
      formula:"Y = q sin(c) + 12eo − o k²/6",
      explain:"中间略收、两侧略弯。现在是完整浮蚕。",
      plot:"final", axis:"⟨ q+40cos(c) , q sin c − ok²/6 + 12eo ⟩" },
    { ch:"动画", title:"t 一直增加，浮蚕游起来",
      formula:"t ← t + π/90　→　c 和 q₂ 都在变",
      explain:"朝向和幅度同时轻轻改。点「播放游动」可暂停。",
      plot:"final", axis:"完整坐标", loopT:true, lockView:true }
  ];

  var NEED_Q = {
    u_e:1, q1_e:1, q2_e:1, q_e:1, rot:1, x_full_yrot:1, y_spine:1, final:1
  };

  // 固定两个示范点，标在图上（避开 k=0）
  var PIN_A = 8020; // x=20, y=80
  var PIN_B = 8080; // x=80, y=80

  var step = 0, t = 1.2, morph = 1, teaching = false, teachWait = 0;
  var view = {x0:-1,x1:1,y0:-1,y1:1};
  var LAST = STEPS.length - 1;
  var lastTs = 0;
  var pauseLoop = false;

  var cv = document.getElementById("cv");
  var ctx = cv.getContext("2d", { alpha: false });

  function ease(u) {
    u = Math.max(0, Math.min(1, u));
    return u * u * (3 - 2 * u);
  }

  function valsAt(i, tNow) {
    var x = xs[i], y = ys[i];
    var k = x / 4 - 12.5;
    var e = y / 9 + 6;
    var o = Math.sqrt(k * k + e * e) / 9;
    var c = o / 2 + e / 2 - tNow / 4;
    var ht = 0.5 * Math.tan(y / 2);
    var u = ht + Math.cos(y);
    var q1 = (3 / k) * u;
    var q2 = k * (5 / o + o * Math.sin(y) * Math.sin(e + 4 * o - tNow));
    var q = q1 + q2;
    var Xrot = 40 * Math.cos(c);
    var Yrot = q * Math.sin(c);
    return {
      x:x, y:y, k:k, e:e, o:o, c:c, u:u, q1:q1, q2:q2, q:q,
      Xrot:Xrot, Yrot:Yrot, X:q+Xrot, Y:Yrot+12*e*o-(o*k*k)/6
    };
  }

  function fmt(n, d) {
    if (!isFinite(n)) return "?";
    return (Math.round(n * Math.pow(10, d)) / Math.pow(10, d)).toFixed(d);
  }

  function pinText(S, v, which) {
    var p = S.plot;
    if (p === "x_only") return which + "  x=" + fmt(v.x, 0);
    if (p === "xy") return which + "  (x,y)=(" + fmt(v.x,0) + "," + fmt(v.y,0) + ")";
    if (p === "k_y") return which + "  k=" + fmt(v.k, 2);
    if (p === "x_e") return which + "  e=" + fmt(v.e, 2);
    if (p === "ke") return which + "  (k,e)=(" + fmt(v.k,1) + "," + fmt(v.e,1) + ")";
    if (p === "k_o") return which + "  o=" + fmt(v.o, 2);
    if (p === "o2_e") return which + "  o/2=" + fmt(v.o/2, 2);
    if (p === "c_no_t") return which + "  o/2+e/2=" + fmt(v.o/2+v.e/2, 2);
    if (p === "c_e") return which + "  c=" + fmt(v.c, 2);
    if (p === "u_e") return which + "  u=" + fmt(v.u, 2);
    if (p === "q1_e") return which + "  q₁=" + fmt(v.q1, 1);
    if (p === "q2_e") return which + "  q₂=" + fmt(v.q2, 1);
    if (p === "q_e") return which + "  q=" + fmt(v.q, 1);
    if (p === "rot") return which + "  转后";
    if (p === "x_full_yrot") return which + "  X=" + fmt(v.X, 1);
    if (p === "y_spine") return which + "  拉长后";
    return which + "  最终点";
  }

  function wrapLines(ctx, text, maxW, maxLines) {
    var out = [], line = "";
    for (var i = 0; i < text.length; i++) {
      var t2 = line + text[i];
      if (ctx.measureText(t2).width > maxW && line) {
        out.push(line);
        line = text[i];
        if (maxLines && out.length >= maxLines) {
          var last = out[out.length - 1];
          while (ctx.measureText(last + "…").width > maxW && last.length) last = last.slice(0, -1);
          out[out.length - 1] = last + "…";
          return out;
        }
      } else line = t2;
    }
    if (line) out.push(line);
    return out;
  }

  function measureLayout(ctx, w, h, S) {
    var topEl = document.getElementById("lessonTop");
    var botEl = document.getElementById("lessonBot");
    var dpr = w / Math.max(1, window.innerWidth);
    var topH = Math.ceil(topEl.getBoundingClientRect().bottom * dpr) + 8;
    var botH = Math.ceil((window.innerHeight - botEl.getBoundingClientRect().top) * dpr) + 8;
    return { topH: topH, botH: botH };
  }

  function drawDock(ctx, x, y, lines, alpha, fs) {
    var pad = 6, lh = fs + 4;
    ctx.font = fs + "px sans-serif";
    var tw = 0;
    for (var i = 0; i < lines.length; i++) tw = Math.max(tw, ctx.measureText(lines[i]).width);
    var bw = tw + pad * 2, bh = lines.length * lh + pad;
    ctx.fillStyle = "rgba(8,8,8," + (0.88 * alpha) + ")";
    ctx.strokeStyle = "rgba(255,224,138," + (0.75 * alpha) + ")";
    ctx.lineWidth = 1.2;
    ctx.beginPath();
    if (ctx.roundRect) ctx.roundRect(x, y, bw, bh, 6);
    else ctx.rect(x, y, bw, bh);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "rgba(255,244,200," + alpha + ")";
    ctx.textAlign = "left";
    ctx.textBaseline = "top";
    for (i = 0; i < lines.length; i++) ctx.fillText(lines[i], x + pad, y + pad + i * lh);
    return {w: bw, h: bh};
  }

  function drawOverlay(ctx, w, h, S, mapX, mapY, alpha, L) {
    ctx.save();
    ctx.beginPath();
    ctx.rect(0, L.topH, w, Math.max(8, h - L.topH - L.botH));
    ctx.clip();
    ctx.globalAlpha = 0.28 * alpha;
    ctx.strokeStyle = "#ffe08a";
    ctx.lineWidth = 1;
    ctx.setLineDash([5, 5]);
    if (view.x0 < 0 && view.x1 > 0) {
      var zx = mapX(0);
      ctx.beginPath(); ctx.moveTo(zx, L.topH); ctx.lineTo(zx, h - L.botH); ctx.stroke();
    }
    if (view.y0 < 0 && view.y1 > 0) {
      var zy = mapY(0);
      ctx.beginPath(); ctx.moveTo(0, zy); ctx.lineTo(w, zy); ctx.stroke();
    }
    ctx.restore();

    var fsPin = Math.max(11, Math.min(14, (Math.min(w, 900) / 38) | 0));
    var dockY = L.topH + 8;
    var plotBottom = h - L.botH - 8;
    var va = valsAt(PIN_A, t), vb = valsAt(PIN_B, t);
    var showA = aliveTo[PIN_A] || aliveFrom[PIN_A];
    var showB = aliveTo[PIN_B] || aliveFrom[PIN_B];
    ctx.font = fsPin + "px sans-serif";

    function leader(sx, sy, boxX, boxY, boxW, boxH) {
      sy = Math.max(L.topH + 6, Math.min(plotBottom, sy));
      sx = Math.max(6, Math.min(w - 6, sx));
      ctx.strokeStyle = "rgba(255,224,138," + (0.8 * alpha) + ")";
      ctx.lineWidth = 1.2;
      ctx.beginPath();
      ctx.moveTo(sx, sy);
      ctx.lineTo(boxX + boxW / 2, boxY + boxH);
      ctx.stroke();
      ctx.beginPath();
      ctx.fillStyle = "rgba(255,224,80," + alpha + ")";
      ctx.arc(sx, sy, 4.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 1;
      ctx.stroke();
    }

    if (showA) {
      var boxA = drawDock(ctx, 8, dockY, [pinText(S, va, "①")], alpha, fsPin);
      leader(mapX(px[PIN_A]), mapY(py[PIN_A]), 8, dockY, boxA.w, boxA.h);
    }
    if (showB) {
      ctx.font = fsPin + "px sans-serif";
      var tw = ctx.measureText(pinText(S, vb, "②")).width + 12;
      var bx = w - 8 - tw;
      var dockBY = (showA && bx < 8 + 160) ? dockY + 28 : dockY;
      if (dockBY + 28 > plotBottom) dockBY = L.topH + 8;
      var boxB = drawDock(ctx, bx, dockBY, [pinText(S, vb, "②")], alpha, fsPin);
      leader(mapX(px[PIN_B]), mapY(py[PIN_B]), bx, dockBY, boxB.w, boxB.h);
    }
  }

  function fillPlot(plot, tNow, outX, outY, alive) {
    var needQ = !!NEED_Q[plot];
    for (var i = 0; i < N; i++) {
      var x = xs[i], y = ys[i];
      var k = x / 4 - 12.5;
      var e = y / 9 + 6;
      var o = Math.sqrt(k * k + e * e) / 9;
      var ok = isFinite(k) && isFinite(e) && isFinite(o) && Math.abs(k) > 1e-9 && o > 1e-9;
      var u = 0, q1 = 0, q2 = 0, q = 0, c = 0, Xrot = 0, Yrot = 0, X = 0, Y = 0;
      if (ok) {
        c = o / 2 + e / 2 - tNow / 4;
        if (needQ) {
          if (Math.abs(Math.cos(y / 2)) < 0.015) ok = false;
          else {
            var ht = 0.5 * Math.tan(y / 2);
            if (!isFinite(ht) || Math.abs(ht) > 60) ok = false;
            else {
              u = ht + Math.cos(y);
              q1 = (3 / k) * u;
              q2 = k * (5 / o + o * Math.sin(y) * Math.sin(e + 4 * o - tNow));
              q = q1 + q2;
              Xrot = 40 * Math.cos(c);
              Yrot = q * Math.sin(c);
              X = q + Xrot;
              Y = Yrot + 12 * e * o - (o * k * k) / 6;
              if (!isFinite(q) || !isFinite(X) || !isFinite(Y)) ok = false;
            }
          }
        }
      }
      if (!ok) { alive[i] = 0; continue; }
      alive[i] = 1;
      if (plot === "x_only") { outX[i] = x; outY[i] = 0; }
      else if (plot === "xy") { outX[i] = x; outY[i] = y; }
      else if (plot === "k_y") { outX[i] = k; outY[i] = y; }
      else if (plot === "x_e") { outX[i] = x; outY[i] = e; }
      else if (plot === "ke") { outX[i] = k; outY[i] = e; }
      else if (plot === "k_o") { outX[i] = k; outY[i] = o; }
      else if (plot === "o2_e") { outX[i] = o / 2; outY[i] = e; }
      else if (plot === "c_no_t") { outX[i] = o / 2 + e / 2; outY[i] = e; }
      else if (plot === "c_e") { outX[i] = c; outY[i] = e; }
      else if (plot === "u_e") { outX[i] = u; outY[i] = e; }
      else if (plot === "q1_e") { outX[i] = q1; outY[i] = e; }
      else if (plot === "q2_e") { outX[i] = q2; outY[i] = e; }
      else if (plot === "q_e") { outX[i] = q; outY[i] = e; }
      else if (plot === "rot") { outX[i] = Xrot; outY[i] = Yrot; }
      else if (plot === "x_full_yrot") { outX[i] = X; outY[i] = Yrot; }
      else if (plot === "y_spine") { outX[i] = X; outY[i] = Yrot + 12 * e * o; }
      else { outX[i] = X; outY[i] = Y; }
    }
  }

  function boundsOf(ax, ay, alive, extra) {
    var xs2 = [], ys2 = [];
    for (var i = 0; i < N; i++) {
      if (!alive[i]) continue;
      if (!isFinite(ax[i]) || !isFinite(ay[i])) continue;
      xs2.push(ax[i]); ys2.push(ay[i]);
    }
    if (xs2.length < 8) return {x0:-1,x1:1,y0:-1,y1:1};
    xs2.sort(function (a,b){return a-b;});
    ys2.sort(function (a,b){return a-b;});
    var a = (xs2.length * 0.02) | 0, b = Math.min(xs2.length - 1, (xs2.length * 0.98) | 0);
    var x0 = xs2[a], x1 = xs2[b], y0 = ys2[a], y1 = ys2[b];
    if (extra) {
      x0 = Math.min(x0, extra.x0); x1 = Math.max(x1, extra.x1);
      y0 = Math.min(y0, extra.y0); y1 = Math.max(y1, extra.y1);
    }
    if (x1 - x0 < 1e-3) { x0 -= 1; x1 += 1; }
    if (y1 - y0 < 1e-3) { y0 -= 1; y1 += 1; }
    var gx = (x1 - x0) * 0.12, gy = (y1 - y0) * 0.12;
    return {x0:x0-gx, x1:x1+gx, y0:y0-gy, y1:y1+gy};
  }

  function enterStep(n) {
    n = Math.max(0, Math.min(LAST, n));
    fromX.set(toX); fromY.set(toY); aliveFrom.set(aliveTo);
    step = n;
    var S = STEPS[step];
    fillPlot(S.plot, t, toX, toY, aliveTo);
    morph = 0;
    if (S.lockView && S.loopT) {
      // 预留 t 滑动空间：用当前 to 的视野再左右加宽
      view = boundsOf(toX, toY, aliveTo);
      var span = view.x1 - view.x0;
      view.x0 -= span * 0.35;
      view.x1 += span * 0.35;
    } else {
      view = boundsOf(toX, toY, aliveTo, boundsOf(fromX, fromY, aliveFrom));
    }
    document.getElementById("btnPrev").disabled = step === 0;
    document.getElementById("btnNext").disabled = step === LAST;
    document.getElementById("bar").style.width = ((step + 1) / STEPS.length * 100) + "%";
    document.getElementById("meta").textContent = "第 " + (step + 1) + " / " + STEPS.length + " 步 · " + S.ch;
    document.getElementById("ltitle").textContent = S.title;
    document.getElementById("lformula").textContent = S.formula;
    document.getElementById("lexplain").textContent = S.explain;
    document.getElementById("laxis").textContent = S.axis || "";
    teachWait = 0;
  }

  function drawFrame(ts) {
    if (!lastTs) lastTs = ts;
    var dt = Math.min(0.05, (ts - lastTs) / 1000);
    lastTs = ts;

    var S = STEPS[step];
    if (morph < 1) morph = Math.min(1, morph + dt / 0.85);

    if (morph >= 1 && S.loopT && !pauseLoop) {
      t += Math.PI / 90 * (dt * 60 / 1.0);
      fillPlot(S.plot, t, toX, toY, aliveTo);
      fromX.set(toX); fromY.set(toY); aliveFrom.set(aliveTo);
      document.getElementById("meta").textContent = "第 " + (step + 1) + " / " + STEPS.length + " 步 · " + S.ch + " · t=" + fmt(t, 2);
    }

    var m = ease(morph);
    for (var i = 0; i < N; i++) {
      var a0 = aliveFrom[i], a1 = aliveTo[i];
      if (!a0 && !a1) continue;
      var x0 = a0 ? fromX[i] : toX[i];
      var y0 = a0 ? fromY[i] : toY[i];
      var x1 = a1 ? toX[i] : fromX[i];
      var y1 = a1 ? toY[i] : fromY[i];
      px[i] = x0 + (x1 - x0) * m;
      py[i] = y0 + (y1 - y0) * m;
    }

    var dpr = Math.min(window.devicePixelRatio || 1, 2);
    var wCss = window.innerWidth;
    var hCss = window.innerHeight;
    if (cv.width !== (wCss * dpr | 0) || cv.height !== (hCss * dpr | 0)) {
      cv.width = wCss * dpr | 0;
      cv.height = hCss * dpr | 0;
    }
    var w = cv.width, h = cv.height;
    ctx.setTransform(1,0,0,1,0,0);
    ctx.fillStyle = "#050505";
    ctx.fillRect(0, 0, w, h);

    var L = measureLayout(ctx, w, h, S);
    var plotTop = L.topH + 6;
    var plotBot = h - L.botH - 6;
    var plotH = Math.max(40, plotBot - plotTop);
    var side = Math.max(20 * dpr, w * 0.03);
    var dx = view.x1 - view.x0, dy = view.y1 - view.y0;
    var innerW = Math.max(40, w - 2 * side);
    var scale = Math.min(innerW / dx, plotH / dy);
    var ox = (w - dx * scale) / 2;
    var oy = plotTop + (plotH - dy * scale) / 2;
    function mapX(x) { return ox + (x - view.x0) * scale; }
    function mapY(y) { return oy + (view.y1 - y) * scale; }
    var s = Math.max(1.4, Math.min(2.8, Math.min(w, h) / 520));
    var finalish = S.plot === "final" || S.plot === "y_spine" || S.plot === "x_full_yrot" || S.plot === "rot";
    ctx.fillStyle = finalish ? "rgba(255,255,255,0.30)" : "rgba(255,236,160,0.42)";

    for (i = 0; i < N; i++) {
      if (!aliveFrom[i] && !aliveTo[i]) continue;
      if (morph < 1 && !aliveTo[i] && m > 0.2) continue;
      var X = px[i], Y = py[i];
      if (X < view.x0 || X > view.x1 || Y < view.y0 || Y > view.y1) continue;
      var ax = mapX(X), ay = mapY(Y);
      if (ay < plotTop || ay > plotBot) continue;
      ctx.fillRect(ax, ay, s, s);
    }

    var ol = 0.4 + 0.6 * m;
    drawOverlay(ctx, w, h, S, mapX, mapY, ol, L);

    if (teaching && morph >= 1) {
      teachWait += dt;
      var need = S.loopT ? 2.4 : 1.15;
      if (teachWait >= need) {
        if (step >= LAST) stopTeach();
        else enterStep(step + 1);
      }
    }
    requestAnimationFrame(drawFrame);
  }

  function stopTeach() {
    teaching = false;
    document.getElementById("btnAuto").textContent = "自动讲解";
  }

  document.getElementById("btnPrev").onclick = function () {
    stopTeach(); pauseLoop = false; enterStep(step - 1);
  };
  document.getElementById("btnNext").onclick = function () {
    stopTeach(); pauseLoop = false; enterStep(step + 1);
  };
  document.getElementById("btnAuto").onclick = function () {
    teaching = !teaching;
    document.getElementById("btnAuto").textContent = teaching ? "停止讲解" : "自动讲解";
    if (teaching) teachWait = 0;
  };
  document.getElementById("btnPlay").onclick = function () {
    stopTeach();
    if (step !== LAST) {
      pauseLoop = false;
      enterStep(LAST);
      document.getElementById("btnPlay").textContent = "暂停游动";
    } else {
      pauseLoop = !pauseLoop;
      document.getElementById("btnPlay").textContent = pauseLoop ? "播放游动" : "暂停游动";
    }
  };
  window.addEventListener("keydown", function (e) {
    if (e.key === "ArrowRight") { stopTeach(); enterStep(step + 1); }
    if (e.key === "ArrowLeft") { stopTeach(); enterStep(step - 1); }
  });
  window.addEventListener("resize", function () { /* 下一帧会重画 */ });

  fillPlot(STEPS[0].plot, t, toX, toY, aliveTo);
  fromX.set(toX); fromY.set(toY); aliveFrom.set(aliveTo);
  enterStep(0);
  morph = 1;
  requestAnimationFrame(drawFrame);
})();
  </script>
</body>
</html>
"""
