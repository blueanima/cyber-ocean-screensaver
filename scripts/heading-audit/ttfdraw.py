"""Minimal TrueType CJK rasterizer for DroidSansFallbackFull.ttf."""
import struct

FONT = "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf"


class Ttf:
    def __init__(self, path=FONT):
        with open(path, "rb") as f:
            self.d = f.read()
        n = struct.unpack(">H", self.d[4:6])[0]
        self.tab = {}
        p = 12
        for _ in range(n):
            tag, _, off, ln = struct.unpack(">4sIII", self.d[p : p + 16])
            self.tab[tag] = (off, ln)
            p += 16
        hs, _ = self.tab[b"head"]
        self.upem = struct.unpack(">H", self.d[hs + 18 : hs + 20])[0]
        self.loca_long = struct.unpack(">h", self.d[hs + 50 : hs + 52])[0] == 1
        ms, _ = self.tab[b"maxp"]
        self.ng = struct.unpack(">H", self.d[ms + 4 : ms + 6])[0]
        self._cmap = self._parse_cmap()
        hh, _ = self.tab[b"hhea"]
        self.n_hmtx = struct.unpack(">h", self.d[hh + 34 : hh + 36])[0]
        self.hmtx0, _ = self.tab[b"hmtx"]

    def u16(self, o):
        return struct.unpack(">H", self.d[o : o + 2])[0]

    def i16(self, o):
        return struct.unpack(">h", self.d[o : o + 2])[0]

    def i32(self, o):
        return struct.unpack(">i", self.d[o : o + 4])[0]

    def u32(self, o):
        return struct.unpack(">I", self.d[o : o + 4])[0]

    def _parse_cmap(self):
        cs, _ = self.tab[b"cmap"]
        nenc = self.u16(cs + 2)
        fmt4 = None
        p = cs + 4
        for _ in range(nenc):
            plat, enc, eoff = struct.unpack(">HHI", self.d[p : p + 8])
            p += 8
            fmt = self.u16(cs + eoff)
            if fmt == 4 and plat == 3:
                fmt4 = cs + eoff
        if fmt4 is None:
            raise RuntimeError("no cmap format 4")
        seg = self.u16(fmt4 + 6) // 2
        end = fmt4 + 14
        start = end + 2 + seg * 2
        delta = start + seg * 2
        rangeoff = delta + seg * 2
        ends = [self.u16(end + i * 2) for i in range(seg)]
        starts = [self.u16(start + i * 2) for i in range(seg)]
        deltas = [self.i16(delta + i * 2) for i in range(seg)]
        roffs = [self.u16(rangeoff + i * 2) for i in range(seg)]
        return ends, starts, deltas, roffs, rangeoff

    def gid(self, cp):
        ends, starts, deltas, roffs, rangeoff = self._cmap
        for i, e in enumerate(ends):
            if cp <= e:
                if cp < starts[i]:
                    return 0
                if roffs[i] == 0:
                    return (cp + deltas[i]) & 0xFFFF
                addr = rangeoff + i * 2 + roffs[i] + (cp - starts[i]) * 2
                g = self.u16(addr)
                if g == 0:
                    return 0
                return (g + deltas[i]) & 0xFFFF
        return 0

    def advance(self, gid):
        if gid < self.n_hmtx:
            return self.u16(self.hmtx0 + gid * 4)
        return self.u16(self.hmtx0 + (self.n_hmtx - 1) * 4)

    def loca(self, gid):
        ls, _ = self.tab[b"loca"]
        gs, _ = self.tab[b"glyf"]
        if self.loca_long:
            return gs + self.u32(ls + gid * 4), gs + self.u32(ls + (gid + 1) * 4)
        return gs + self.u16(ls + gid * 2) * 2, gs + self.u16(ls + (gid + 1) * 2) * 2

    def glyph_pts(self, gid, dx=0, dy=0, sx=1.0, sy=1.0):
        a, b = self.loca(gid)
        if b <= a:
            return []
        ncont = self.i16(a)
        if ncont == 0:
            return []
        if ncont < 0:
            return self._composite(a, dx, dy, sx, sy)
        ends = [self.u16(a + 10 + i * 2) for i in range(ncont)]
        npts = ends[-1] + 1
        ins = self.u16(a + 10 + ncont * 2)
        p = a + 12 + ncont * 2 + ins
        flags = []
        i = 0
        while i < npts:
            fl = self.d[p]
            p += 1
            if fl & 8:
                rep = self.d[p]
                p += 1
                flags.extend([fl] * (rep + 1))
                i += rep + 1
            else:
                flags.append(fl)
                i += 1
        xs = []
        x = 0
        for fl in flags:
            if fl & 2:
                v = self.d[p]
                p += 1
                x += v if (fl & 16) else -v
            elif not (fl & 16):
                x += self.i16(p)
                p += 2
            xs.append(x)
        ys = []
        y = 0
        for fl in flags:
            if fl & 4:
                v = self.d[p]
                p += 1
                y += v if (fl & 32) else -v
            elif not (fl & 32):
                y += self.i16(p)
                p += 2
            ys.append(y)
        pts = [((xs[i] + dx) * sx, (ys[i] + dy) * sy, flags[i] & 1) for i in range(npts)]
        contours = []
        prev = 0
        for e in ends:
            contours.append(pts[prev : e + 1])
            prev = e + 1
        return contours

    def _composite(self, a, dx, dy, sx, sy):
        p = a + 10
        out = []
        ARG_WORDS, ARGS_XY, HAVE_SCALE, MORE, HAVE_XY, HAVE_2X2 = 1, 2, 8, 32, 64, 128
        while True:
            fl = self.u16(p)
            gid = self.u16(p + 2)
            p += 4
            if fl & ARG_WORDS:
                ax, ay = self.i16(p), self.i16(p + 2)
                p += 4
            else:
                ax, ay = struct.unpack("bb", self.d[p : p + 2])
                p += 2
            m00 = m11 = 1.0
            m01 = m10 = 0.0
            if fl & HAVE_SCALE:
                m00 = m11 = self.i16(p) / 16384.0
                p += 2
            elif fl & HAVE_XY:
                m00 = self.i16(p) / 16384.0
                m11 = self.i16(p + 2) / 16384.0
                p += 4
            elif fl & HAVE_2X2:
                m00 = self.i16(p) / 16384.0
                m01 = self.i16(p + 2) / 16384.0
                m10 = self.i16(p + 4) / 16384.0
                m11 = self.i16(p + 6) / 16384.0
                p += 8
            if fl & ARGS_XY:
                ox, oy = ax, ay
            else:
                ox = oy = 0
            sub = self.glyph_pts(gid, 0, 0, 1, 1)
            for cont in sub:
                nc = []
                for x, y, on in cont:
                    nx = x * m00 + y * m10 + ox
                    ny = x * m01 + y * m11 + oy
                    nc.append(((nx + dx) * sx, (ny + dy) * sy, on))
                out.append(nc)
            if not (fl & MORE):
                break
        return out


def flatten(contour, step=0.35):
    if not contour:
        return []
    pts = list(contour)
    # implied on-curve midpoints between two off-curve
    exp = []
    n = len(pts)
    for i in range(n):
        x, y, on = pts[i]
        exp.append((x, y, on))
        x2, y2, on2 = pts[(i + 1) % n]
        if (not on) and (not on2):
            exp.append(((x + x2) * 0.5, (y + y2) * 0.5, 1))
    # walk
    on_i = [i for i, p in enumerate(exp) if p[2]]
    if not on_i:
        return [(p[0], p[1]) for p in exp]
    # rotate so start on-curve
    s = on_i[0]
    exp = exp[s:] + exp[:s]
    out = []
    i = 0
    n = len(exp)
    while i < n:
        x0, y0, on0 = exp[i]
        if not on0:
            i += 1
            continue
        x1, y1, on1 = exp[(i + 1) % n]
        if on1:
            out.append((x0, y0))
            i += 1
            if i >= n:
                break
            continue
        x2, y2, on2 = exp[(i + 2) % n]
        # quadratic x0 -- x1(off) -- x2(on)
        for t in range(0, 8):
            u = t / 8.0
            a, b = (1 - u) ** 2, 2 * (1 - u) * u
            c = u * u
            out.append((a * x0 + b * x1 + c * x2, a * y0 + b * y1 + c * y2))
        i += 2
        if i >= n:
            break
    if out:
        out.append(out[0])
    return out


def fill_mask(contours, w, h, ox, oy, scale):
    # scale font units -> pixels; y up in font, y down in image
    paths = []
    for c in contours:
        poly = flatten(c)
        pix = []
        for x, y in poly:
            pix.append((ox + x * scale, oy - y * scale))
        if len(pix) >= 3:
            paths.append(pix)
    mask = bytearray(w * h)
    if not paths:
        return mask
    # scanline nonzero
    for y in range(h):
        y0 = y + 0.5
        xs = []
        for poly in paths:
            for i in range(len(poly) - 1):
                x1, y1 = poly[i]
                x2, y2 = poly[i + 1]
                if y1 == y2:
                    continue
                if y0 < min(y1, y2) or y0 >= max(y1, y2):
                    continue
                t = (y0 - y1) / (y2 - y1)
                xs.append(x1 + t * (x2 - x1))
        xs.sort()
        for i in range(0, len(xs) - 1, 2):
            a = max(0, int(xs[i]))
            b = min(w - 1, int(xs[i + 1]))
            base = y * w
            for x in range(a, b + 1):
                mask[base + x] = 255
    return mask


_FONT = None


def font():
    global _FONT
    if _FONT is None:
        _FONT = Ttf()
    return _FONT


def measure(text, px):
    f = font()
    sc = px / f.upem
    return int(sum(f.advance(f.gid(ord(ch))) for ch in text) * sc)


def draw_text(put, x, y, text, rgb, px=18):
    """Baseline at y. put(ix,iy,r,g,b,a)."""
    f = font()
    sc = px / f.upem
    pen = 0.0
    for ch in text:
        gid = f.gid(ord(ch))
        adv = f.advance(gid)
        cont = f.glyph_pts(gid)
        if cont:
            # glyph bbox
            xs = [p[0] for c in cont for p in c]
            ys = [p[1] for c in cont for p in c]
            if xs:
                minx, maxx = min(xs), max(xs)
                miny, maxy = min(ys), max(ys)
                gw = max(1, int((maxx - minx) * sc) + 3)
                gh = max(1, int((maxy - miny) * sc) + 3)
                ox = -minx * sc + 1
                oy = maxy * sc + 1
                mask = fill_mask(cont, gw, gh, ox, oy, sc)
                gx = int(x + pen + minx * sc)
                gy = int(y - maxy * sc)
                for row in range(gh):
                    for col in range(gw):
                        if mask[row * gw + col]:
                            put(gx + col, gy + row, rgb[0], rgb[1], rgb[2], 1.0)
        pen += adv * sc
    return pen
