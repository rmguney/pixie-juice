let jn, j, F, An, Fn, D, V, H, N, Y, K, P, Q, J, X, Z, tt, _t, et, nt, ot, rt, it, C, st, gt, ct, at, pt, ft, bt, wt, dt, mt, ut, lt, ht, De, Ye, gn, cn, an, sn, rn, bn, wn, pn, on, Ve, He, Se, kn, _n, In, en, yn, $n, zn, nn, tn, Ge, vn, Je, ln, Xe, Me, Pe, Re, un, xn, Ke, hn, Ze, Qe, Be, We, qe, Oe, Te, Le, Ee, Ne, Ce, dn, mn, fn, Ue;
let __tla = (async ()=>{
    const E = "/assets/pixie_juice_bg-BLy0wKmI.wasm", B = async (e = {}, t)=>{
        let o;
        if (t.startsWith("data:")) {
            const r = t.replace(/^data:.*?base64,/, "");
            let c;
            if (typeof Buffer == "function" && typeof Buffer.from == "function") c = Buffer.from(r, "base64");
            else if (typeof atob == "function") {
                const a = atob(r);
                c = new Uint8Array(a.length);
                for(let g = 0; g < a.length; g++)c[g] = a.charCodeAt(g);
            } else throw new Error("Cannot decode base64-encoded data URL");
            o = await WebAssembly.instantiate(c, e);
        } else {
            const r = await fetch(t), c = r.headers.get("Content-Type") || "";
            if ("instantiateStreaming" in WebAssembly && c.startsWith("application/wasm")) o = await WebAssembly.instantiateStreaming(r, e);
            else {
                const a = await r.arrayBuffer();
                o = await WebAssembly.instantiate(a, e);
            }
        }
        return o.instance.exports;
    };
    let _;
    C = function(e) {
        _ = e;
    };
    const x = new Array(128).fill(void 0);
    x.push(void 0, null, !0, !1);
    function h(e) {
        return x[e];
    }
    let z = x.length;
    function l(e) {
        z === x.length && x.push(x.length + 1);
        const t = z;
        return z = x[t], x[t] = e, t;
    }
    function L(e, t) {
        try {
            return e.apply(this, t);
        } catch (o) {
            _.__wbindgen_export_0(l(o));
        }
    }
    const S = typeof TextDecoder > "u" ? (0, module.require)("util").TextDecoder : TextDecoder;
    let W = new S("utf-8", {
        ignoreBOM: !0,
        fatal: !0
    });
    W.decode();
    let k = null;
    function I() {
        return (k === null || k.byteLength === 0) && (k = new Uint8Array(_.memory.buffer)), k;
    }
    function u(e, t) {
        return e = e >>> 0, W.decode(I().subarray(e, e + t));
    }
    function v(e) {
        return e == null;
    }
    function M(e) {
        e < 132 || (x[e] = z, z = e);
    }
    function d(e) {
        const t = h(e);
        return M(e), t;
    }
    let b = 0;
    const R = typeof TextEncoder > "u" ? (0, module.require)("util").TextEncoder : TextEncoder;
    let $ = new R("utf-8");
    const G = typeof $.encodeInto == "function" ? function(e, t) {
        return $.encodeInto(e, t);
    } : function(e, t) {
        const o = $.encode(e);
        return t.set(o), {
            read: e.length,
            written: o.length
        };
    };
    function U(e, t, o) {
        if (o === void 0) {
            const n = $.encode(e), p = t(n.length, 1) >>> 0;
            return I().subarray(p, p + n.length).set(n), b = n.length, p;
        }
        let r = e.length, c = t(r, 1) >>> 0;
        const a = I();
        let g = 0;
        for(; g < r; g++){
            const n = e.charCodeAt(g);
            if (n > 127) break;
            a[c + g] = n;
        }
        if (g !== r) {
            g !== 0 && (e = e.slice(g)), c = o(c, r, r = g + e.length * 3, 1) >>> 0;
            const n = I().subarray(c + g, c + r), p = G(e, n);
            g += p.written, c = o(c, r, g, 1) >>> 0;
        }
        return b = g, c;
    }
    let y = null;
    function s() {
        return (y === null || y.buffer.detached === !0 || y.buffer.detached === void 0 && y.buffer !== _.memory.buffer) && (y = new DataView(_.memory.buffer)), y;
    }
    function w(e, t) {
        const o = t(e.length * 1, 1) >>> 0;
        return I().set(e, o / 1), b = e.length, o;
    }
    function m(e, t) {
        return e = e >>> 0, I().subarray(e / 1, e / 1 + t);
    }
    Oe = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.pixie_optimize_auto(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Te = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.pixie_optimize_image(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Le = function(e) {
        try {
            const g = _.__wbindgen_add_to_stack_pointer(-16), n = w(e, _.__wbindgen_export_2), p = b;
            _.pixie_optimize_mesh(g, n, p);
            var t = s().getInt32(g + 0, !0), o = s().getInt32(g + 4, !0), r = s().getInt32(g + 8, !0), c = s().getInt32(g + 12, !0);
            if (c) throw d(r);
            var a = m(t, o).slice();
            return _.__wbindgen_export_1(t, o * 1, 1), a;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    We = function() {
        return _.pixie_get_memory_target_mb();
    };
    qe = function() {
        const e = _.get_performance_metrics();
        return d(e);
    };
    Ee = function() {
        _.pixie_reset_performance_stats();
    };
    Be = function() {
        return _.check_performance_compliance() !== 0;
    };
    Ce = function() {
        const e = _.run_wasm_benchmarks();
        return d(e);
    };
    Se = function() {
        _.init();
    };
    Me = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_image(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Re = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_mesh(n, p, f, v(t) ? 4294967297 : Math.fround(t));
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Ge = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_auto(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Ue = function() {
        let e, t;
        try {
            const c = _.__wbindgen_add_to_stack_pointer(-16);
            _.version(c);
            var o = s().getInt32(c + 0, !0), r = s().getInt32(c + 4, !0);
            return e = o, t = r, u(o, r);
        } finally{
            _.__wbindgen_add_to_stack_pointer(16), _.__wbindgen_export_1(e, t, 1);
        }
    };
    De = function() {
        let e, t;
        try {
            const c = _.__wbindgen_add_to_stack_pointer(-16);
            _.build_timestamp(c);
            var o = s().getInt32(c + 0, !0), r = s().getInt32(c + 4, !0);
            return e = o, t = r, u(o, r);
        } finally{
            _.__wbindgen_add_to_stack_pointer(16), _.__wbindgen_export_1(e, t, 1);
        }
    };
    Ve = function(e) {
        let t, o;
        try {
            const a = _.__wbindgen_add_to_stack_pointer(-16), g = w(e, _.__wbindgen_export_2), n = b;
            _.detect_format(a, g, n);
            var r = s().getInt32(a + 0, !0), c = s().getInt32(a + 4, !0);
            return t = r, o = c, u(r, c);
        } finally{
            _.__wbindgen_add_to_stack_pointer(16), _.__wbindgen_export_1(t, o, 1);
        }
    };
    He = function() {
        const e = _.get_performance_metrics();
        return d(e);
    };
    Ne = function() {
        _.pixie_reset_performance_stats();
    };
    Ye = function() {
        return _.check_performance_compliance() !== 0;
    };
    Ke = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_png(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Pe = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_jpeg(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Qe = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_webp(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Je = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_gif(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Xe = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_ico(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    Ze = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_tga(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    tn = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_webp(t, o) !== 0;
    };
    _n = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_gif(t, o) !== 0;
    };
    en = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_ico(t, o) !== 0;
    };
    nn = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_tga(t, o) !== 0;
    };
    on = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.convert_to_webp(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    rn = function(e) {
        try {
            const g = _.__wbindgen_add_to_stack_pointer(-16), n = w(e, _.__wbindgen_export_2), p = b;
            _.convert_to_png(g, n, p);
            var t = s().getInt32(g + 0, !0), o = s().getInt32(g + 4, !0), r = s().getInt32(g + 8, !0), c = s().getInt32(g + 12, !0);
            if (c) throw d(r);
            var a = m(t, o).slice();
            return _.__wbindgen_export_1(t, o * 1, 1), a;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    sn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.convert_to_jpeg(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    gn = function(e) {
        try {
            const g = _.__wbindgen_add_to_stack_pointer(-16), n = w(e, _.__wbindgen_export_2), p = b;
            _.convert_to_bmp(g, n, p);
            var t = s().getInt32(g + 0, !0), o = s().getInt32(g + 4, !0), r = s().getInt32(g + 8, !0), c = s().getInt32(g + 12, !0);
            if (c) throw d(r);
            var a = m(t, o).slice();
            return _.__wbindgen_export_1(t, o * 1, 1), a;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    cn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.convert_to_gif(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    an = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.convert_to_ico(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    pn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.convert_to_tiff(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    fn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.strip_tiff_metadata_simd(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    bn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.convert_to_svg(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    wn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.convert_to_tga(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    dn = function(e) {
        const t = _.set_lossless_mode(e);
        return d(t);
    };
    mn = function(e) {
        const t = _.set_preserve_metadata(e);
        return d(t);
    };
    un = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_obj(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    ln = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_gltf(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    hn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_stl(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    vn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_fbx(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    xn = function(e, t) {
        try {
            const n = _.__wbindgen_add_to_stack_pointer(-16), p = w(e, _.__wbindgen_export_2), f = b;
            _.optimize_ply(n, p, f, t);
            var o = s().getInt32(n + 0, !0), r = s().getInt32(n + 4, !0), c = s().getInt32(n + 8, !0), a = s().getInt32(n + 12, !0);
            if (a) throw d(c);
            var g = m(o, r).slice();
            return _.__wbindgen_export_1(o, r * 1, 1), g;
        } finally{
            _.__wbindgen_add_to_stack_pointer(16);
        }
    };
    yn = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_obj(t, o) !== 0;
    };
    In = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_gltf(t, o) !== 0;
    };
    zn = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_stl(t, o) !== 0;
    };
    kn = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_fbx(t, o) !== 0;
    };
    $n = function(e) {
        const t = w(e, _.__wbindgen_export_2), o = b;
        return _.is_ply(t, o) !== 0;
    };
    let A;
    jn = Object.freeze({
        RGB: 0,
        0: "RGB",
        RGBA: 1,
        1: "RGBA",
        Grayscale: 2,
        2: "Grayscale",
        GrayscaleAlpha: 3,
        3: "GrayscaleAlpha",
        CMYK: 4,
        4: "CMYK",
        YUV: 5,
        5: "YUV",
        HSV: 6,
        6: "HSV",
        LAB: 7,
        7: "LAB"
    });
    Fn = Object.freeze({
        QuadricErrorMetrics: 0,
        0: "QuadricErrorMetrics",
        EdgeCollapse: 1,
        1: "EdgeCollapse",
        VertexClustering: 2,
        2: "VertexClustering"
    });
    A = typeof FinalizationRegistry > "u" ? {
        register: ()=>{},
        unregister: ()=>{}
    } : new FinalizationRegistry((e)=>_.__wbg_imageoptconfig_free(e >>> 0, 1));
    j = class {
        static __wrap(t) {
            t = t >>> 0;
            const o = Object.create(j.prototype);
            return o.__wbg_ptr = t, A.register(o, o.__wbg_ptr, o), o;
        }
        __destroy_into_raw() {
            const t = this.__wbg_ptr;
            return this.__wbg_ptr = 0, A.unregister(this), t;
        }
        free() {
            const t = this.__destroy_into_raw();
            _.__wbg_imageoptconfig_free(t, 0);
        }
        get quality() {
            return _.__wbg_get_imageoptconfig_quality(this.__wbg_ptr);
        }
        set quality(t) {
            _.__wbg_set_imageoptconfig_quality(this.__wbg_ptr, t);
        }
        get lossless() {
            return _.__wbg_get_imageoptconfig_lossless(this.__wbg_ptr) !== 0;
        }
        set lossless(t) {
            _.__wbg_set_imageoptconfig_lossless(this.__wbg_ptr, t);
        }
        get preserve_metadata() {
            return _.__wbg_get_imageoptconfig_preserve_metadata(this.__wbg_ptr) !== 0;
        }
        set preserve_metadata(t) {
            _.__wbg_set_imageoptconfig_preserve_metadata(this.__wbg_ptr, t);
        }
        get optimize_colors() {
            return _.__wbg_get_imageoptconfig_optimize_colors(this.__wbg_ptr) !== 0;
        }
        set optimize_colors(t) {
            _.__wbg_set_imageoptconfig_optimize_colors(this.__wbg_ptr, t);
        }
        get max_colors() {
            const t = _.__wbg_get_imageoptconfig_max_colors(this.__wbg_ptr);
            return t === 16777215 ? void 0 : t;
        }
        set max_colors(t) {
            _.__wbg_set_imageoptconfig_max_colors(this.__wbg_ptr, v(t) ? 16777215 : t);
        }
        get use_c_hotspots() {
            return _.__wbg_get_imageoptconfig_use_c_hotspots(this.__wbg_ptr) !== 0;
        }
        set use_c_hotspots(t) {
            _.__wbg_set_imageoptconfig_use_c_hotspots(this.__wbg_ptr, t);
        }
        get enable_simd() {
            return _.__wbg_get_imageoptconfig_enable_simd(this.__wbg_ptr) !== 0;
        }
        set enable_simd(t) {
            _.__wbg_set_imageoptconfig_enable_simd(this.__wbg_ptr, t);
        }
        get compression_level() {
            const t = _.__wbg_get_imageoptconfig_compression_level(this.__wbg_ptr);
            return t === 16777215 ? void 0 : t;
        }
        set compression_level(t) {
            _.__wbg_set_imageoptconfig_compression_level(this.__wbg_ptr, v(t) ? 16777215 : t);
        }
        get fast_mode() {
            return _.__wbg_get_imageoptconfig_fast_mode(this.__wbg_ptr) !== 0;
        }
        set fast_mode(t) {
            _.__wbg_set_imageoptconfig_fast_mode(this.__wbg_ptr, t);
        }
        get preserve_alpha() {
            return _.__wbg_get_imageoptconfig_preserve_alpha(this.__wbg_ptr) !== 0;
        }
        set preserve_alpha(t) {
            _.__wbg_set_imageoptconfig_preserve_alpha(this.__wbg_ptr, t);
        }
        get max_width() {
            const t = _.__wbg_get_imageoptconfig_max_width(this.__wbg_ptr);
            return t === 4294967297 ? void 0 : t;
        }
        set max_width(t) {
            _.__wbg_set_imageoptconfig_max_width(this.__wbg_ptr, v(t) ? 4294967297 : t >>> 0);
        }
        get max_height() {
            const t = _.__wbg_get_imageoptconfig_max_height(this.__wbg_ptr);
            return t === 4294967297 ? void 0 : t;
        }
        set max_height(t) {
            _.__wbg_set_imageoptconfig_max_height(this.__wbg_ptr, v(t) ? 4294967297 : t >>> 0);
        }
        get target_reduction() {
            const t = _.__wbg_get_imageoptconfig_target_reduction(this.__wbg_ptr);
            return t === 4294967297 ? void 0 : t;
        }
        set target_reduction(t) {
            _.__wbg_set_imageoptconfig_target_reduction(this.__wbg_ptr, v(t) ? 4294967297 : Math.fround(t));
        }
    };
    const O = typeof FinalizationRegistry > "u" ? {
        register: ()=>{},
        unregister: ()=>{}
    } : new FinalizationRegistry((e)=>_.__wbg_meshoptconfig_free(e >>> 0, 1));
    F = class {
        static __wrap(t) {
            t = t >>> 0;
            const o = Object.create(F.prototype);
            return o.__wbg_ptr = t, O.register(o, o.__wbg_ptr, o), o;
        }
        __destroy_into_raw() {
            const t = this.__wbg_ptr;
            return this.__wbg_ptr = 0, O.unregister(this), t;
        }
        free() {
            const t = this.__destroy_into_raw();
            _.__wbg_meshoptconfig_free(t, 0);
        }
        get target_ratio() {
            return _.__wbg_get_meshoptconfig_target_ratio(this.__wbg_ptr);
        }
        set target_ratio(t) {
            _.__wbg_set_meshoptconfig_target_ratio(this.__wbg_ptr, t);
        }
        get preserve_topology() {
            return _.__wbg_get_meshoptconfig_preserve_topology(this.__wbg_ptr) !== 0;
        }
        set preserve_topology(t) {
            _.__wbg_set_meshoptconfig_preserve_topology(this.__wbg_ptr, t);
        }
        get weld_vertices() {
            return _.__wbg_get_meshoptconfig_weld_vertices(this.__wbg_ptr) !== 0;
        }
        set weld_vertices(t) {
            _.__wbg_set_meshoptconfig_weld_vertices(this.__wbg_ptr, t);
        }
        get vertex_tolerance() {
            return _.__wbg_get_meshoptconfig_vertex_tolerance(this.__wbg_ptr);
        }
        set vertex_tolerance(t) {
            _.__wbg_set_meshoptconfig_vertex_tolerance(this.__wbg_ptr, t);
        }
        get simplification_algorithm() {
            return _.__wbg_get_meshoptconfig_simplification_algorithm(this.__wbg_ptr);
        }
        set simplification_algorithm(t) {
            _.__wbg_set_meshoptconfig_simplification_algorithm(this.__wbg_ptr, t);
        }
        get use_c_hotspots() {
            return _.__wbg_get_meshoptconfig_use_c_hotspots(this.__wbg_ptr) !== 0;
        }
        set use_c_hotspots(t) {
            _.__wbg_set_meshoptconfig_use_c_hotspots(this.__wbg_ptr, t);
        }
        get generate_normals() {
            return _.__wbg_get_meshoptconfig_generate_normals(this.__wbg_ptr) !== 0;
        }
        set generate_normals(t) {
            _.__wbg_set_meshoptconfig_generate_normals(this.__wbg_ptr, t);
        }
        get optimize_vertex_cache() {
            return _.__wbg_get_meshoptconfig_optimize_vertex_cache(this.__wbg_ptr) !== 0;
        }
        set optimize_vertex_cache(t) {
            _.__wbg_set_meshoptconfig_optimize_vertex_cache(this.__wbg_ptr, t);
        }
        get preserve_uv_seams() {
            return _.__wbg_get_meshoptconfig_preserve_uv_seams(this.__wbg_ptr) !== 0;
        }
        set preserve_uv_seams(t) {
            _.__wbg_set_meshoptconfig_preserve_uv_seams(this.__wbg_ptr, t);
        }
        get preserve_boundaries() {
            return _.__wbg_get_meshoptconfig_preserve_boundaries(this.__wbg_ptr) !== 0;
        }
        set preserve_boundaries(t) {
            _.__wbg_set_meshoptconfig_preserve_boundaries(this.__wbg_ptr, t);
        }
    };
    const T = typeof FinalizationRegistry > "u" ? {
        register: ()=>{},
        unregister: ()=>{}
    } : new FinalizationRegistry((e)=>_.__wbg_pixieconfig_free(e >>> 0, 1));
    An = class {
        __destroy_into_raw() {
            const t = this.__wbg_ptr;
            return this.__wbg_ptr = 0, T.unregister(this), t;
        }
        free() {
            const t = this.__destroy_into_raw();
            _.__wbg_pixieconfig_free(t, 0);
        }
        constructor(){
            const t = _.pixieconfig_new();
            return this.__wbg_ptr = t >>> 0, T.register(this, this.__wbg_ptr, this), this;
        }
        get use_c_hotspots() {
            return _.pixieconfig_use_c_hotspots(this.__wbg_ptr) !== 0;
        }
        set use_c_hotspots(t) {
            _.pixieconfig_set_use_c_hotspots(this.__wbg_ptr, t);
        }
        get quality() {
            return _.pixieconfig_quality(this.__wbg_ptr);
        }
        set quality(t) {
            _.pixieconfig_set_quality(this.__wbg_ptr, t);
        }
        get enable_threading() {
            return _.pixieconfig_enable_threading(this.__wbg_ptr) !== 0;
        }
        set enable_threading(t) {
            _.pixieconfig_set_enable_threading(this.__wbg_ptr, t);
        }
        to_image_config() {
            const t = _.pixieconfig_to_image_config(this.__wbg_ptr);
            return j.__wrap(t);
        }
        to_mesh_config() {
            const t = _.pixieconfig_to_mesh_config(this.__wbg_ptr);
            return F.__wrap(t);
        }
    };
    D = function() {
        return L(function(e, t) {
            const o = h(e).call(h(t));
            return l(o);
        }, arguments);
    };
    V = function(e, t) {
        let o, r;
        try {
            o = e, r = t, console.error(u(e, t));
        } finally{
            _.__wbindgen_export_1(o, r, 1);
        }
    };
    H = function(e) {
        let t;
        try {
            t = h(e) instanceof Window;
        } catch  {
            t = !1;
        }
        return t;
    };
    N = function(e, t, o, r, c, a, g, n) {
        let p, f;
        try {
            p = e, f = t, console.log(u(e, t), u(o, r), u(c, a), u(g, n));
        } finally{
            _.__wbindgen_export_1(p, f, 1);
        }
    };
    Y = function(e, t) {
        console.log(u(e, t));
    };
    K = function(e) {
        console.log(h(e));
    };
    P = function(e, t) {
        let o, r;
        try {
            o = e, r = t, console.log(u(e, t));
        } finally{
            _.__wbindgen_export_1(o, r, 1);
        }
    };
    Q = function(e, t) {
        performance.mark(u(e, t));
    };
    J = function() {
        return L(function(e, t, o, r) {
            let c, a, g, n;
            try {
                c = e, a = t, g = o, n = r, performance.measure(u(e, t), u(o, r));
            } finally{
                _.__wbindgen_export_1(c, a, 1), _.__wbindgen_export_1(g, n, 1);
            }
        }, arguments);
    };
    X = function() {
        const e = new Object;
        return l(e);
    };
    Z = function() {
        const e = new Array;
        return l(e);
    };
    tt = function() {
        const e = new Error;
        return l(e);
    };
    _t = function(e, t) {
        const o = new Function(u(e, t));
        return l(o);
    };
    et = function() {
        return performance.now();
    };
    nt = function(e) {
        return h(e).now();
    };
    ot = function(e) {
        const t = h(e).performance;
        return v(t) ? 0 : l(t);
    };
    rt = function(e, t, o) {
        h(e)[t >>> 0] = d(o);
    };
    it = function(e, t, o) {
        h(e)[d(t)] = d(o);
    };
    st = function(e, t) {
        const o = h(t).stack, r = U(o, _.__wbindgen_export_2, _.__wbindgen_export_3), c = b;
        s().setInt32(e + 4, c, !0), s().setInt32(e + 0, r, !0);
    };
    gt = function() {
        const e = typeof global > "u" ? null : global;
        return v(e) ? 0 : l(e);
    };
    ct = function() {
        const e = typeof globalThis > "u" ? null : globalThis;
        return v(e) ? 0 : l(e);
    };
    at = function() {
        const e = typeof self > "u" ? null : self;
        return v(e) ? 0 : l(e);
    };
    pt = function() {
        const e = typeof window > "u" ? null : window;
        return v(e) ? 0 : l(e);
    };
    ft = function(e) {
        const t = BigInt.asUintN(64, e);
        return l(t);
    };
    bt = function(e, t) {
        const o = new Error(u(e, t));
        return l(o);
    };
    wt = function(e) {
        return h(e) === void 0;
    };
    dt = function(e) {
        return l(e);
    };
    mt = function(e) {
        const t = h(e);
        return l(t);
    };
    ut = function(e) {
        d(e);
    };
    lt = function(e, t) {
        const o = u(e, t);
        return l(o);
    };
    ht = function(e, t) {
        throw new Error(u(e, t));
    };
    URL = globalThis.URL;
    const i = await B({
        "./pixie_juice_bg.js": {
            __wbindgen_error_new: bt,
            __wbindgen_number_new: dt,
            __wbindgen_bigint_from_u64: ft,
            __wbg_set_3f1d0b984ed272ed: it,
            __wbg_now_7f43f6c42e10de17: et,
            __wbg_log_8a185c59294f9f1f: Y,
            __wbg_performance_c185c0cdc2766575: ot,
            __wbindgen_object_drop_ref: ut,
            __wbg_now_d18023d54d4e5500: nt,
            __wbindgen_string_new: lt,
            __wbg_log_c222819a41e063d3: K,
            __wbg_new_78feb108b6472713: Z,
            __wbg_new_405e22f390576ce2: X,
            __wbg_set_37837023f3d740e8: rt,
            __wbg_new_8a6f238a6ece86ea: tt,
            __wbg_stack_0ed75d68575b0f3c: st,
            __wbg_error_7534b8e9a36f1ab4: V,
            __wbindgen_object_clone_ref: mt,
            __wbindgen_is_undefined: wt,
            __wbg_newnoargs_105ed471475aaf50: _t,
            __wbg_call_672a4d21634d4a24: D,
            __wbg_static_accessor_GLOBAL_88a902d13a557d07: gt,
            __wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0: ct,
            __wbg_static_accessor_WINDOW_5de37043a91a9c40: pt,
            __wbg_static_accessor_SELF_37c5d418e4bf5819: at,
            __wbg_measure_fb7825c11612c823: J,
            __wbg_mark_7438147ce31e9d4b: Q,
            __wbg_log_cb9e190acc5753fb: P,
            __wbg_log_0cc1b7768397bcfe: N,
            __wbindgen_throw: ht,
            __wbg_instanceof_Window_def73ea0955fc569: H
        }
    }, E), vt = i.memory, xt = i.__wbg_pixieconfig_free, yt = i.pixieconfig_new, It = i.pixieconfig_use_c_hotspots, zt = i.pixieconfig_set_use_c_hotspots, kt = i.pixieconfig_quality, $t = i.pixieconfig_set_quality, jt = i.pixieconfig_enable_threading, Ft = i.pixieconfig_set_enable_threading, At = i.pixieconfig_to_image_config, Ot = i.pixieconfig_to_mesh_config, Tt = i.__wbg_imageoptconfig_free, Lt = i.__wbg_get_imageoptconfig_quality, Wt = i.__wbg_set_imageoptconfig_quality, qt = i.__wbg_get_imageoptconfig_lossless, Et = i.__wbg_set_imageoptconfig_lossless, Bt = i.__wbg_get_imageoptconfig_preserve_metadata, Ct = i.__wbg_set_imageoptconfig_preserve_metadata, St = i.__wbg_get_imageoptconfig_optimize_colors, Mt = i.__wbg_set_imageoptconfig_optimize_colors, Rt = i.__wbg_get_imageoptconfig_max_colors, Gt = i.__wbg_set_imageoptconfig_max_colors, Ut = i.__wbg_get_imageoptconfig_use_c_hotspots, Dt = i.__wbg_set_imageoptconfig_use_c_hotspots, Vt = i.__wbg_get_imageoptconfig_enable_simd, Ht = i.__wbg_set_imageoptconfig_enable_simd, Nt = i.__wbg_get_imageoptconfig_compression_level, Yt = i.__wbg_set_imageoptconfig_compression_level, Kt = i.__wbg_get_imageoptconfig_fast_mode, Pt = i.__wbg_set_imageoptconfig_fast_mode, Qt = i.__wbg_get_imageoptconfig_preserve_alpha, Jt = i.__wbg_set_imageoptconfig_preserve_alpha, Xt = i.__wbg_get_imageoptconfig_max_width, Zt = i.__wbg_set_imageoptconfig_max_width, t_ = i.__wbg_get_imageoptconfig_max_height, __ = i.__wbg_set_imageoptconfig_max_height, e_ = i.__wbg_get_imageoptconfig_target_reduction, n_ = i.__wbg_set_imageoptconfig_target_reduction, o_ = i.__wbg_meshoptconfig_free, r_ = i.__wbg_get_meshoptconfig_target_ratio, i_ = i.__wbg_set_meshoptconfig_target_ratio, s_ = i.__wbg_get_meshoptconfig_preserve_topology, g_ = i.__wbg_set_meshoptconfig_preserve_topology, c_ = i.__wbg_get_meshoptconfig_weld_vertices, a_ = i.__wbg_set_meshoptconfig_weld_vertices, p_ = i.__wbg_get_meshoptconfig_vertex_tolerance, f_ = i.__wbg_set_meshoptconfig_vertex_tolerance, b_ = i.__wbg_get_meshoptconfig_simplification_algorithm, w_ = i.__wbg_set_meshoptconfig_simplification_algorithm, d_ = i.__wbg_get_meshoptconfig_use_c_hotspots, m_ = i.__wbg_set_meshoptconfig_use_c_hotspots, u_ = i.__wbg_get_meshoptconfig_generate_normals, l_ = i.__wbg_set_meshoptconfig_generate_normals, h_ = i.__wbg_get_meshoptconfig_optimize_vertex_cache, v_ = i.__wbg_set_meshoptconfig_optimize_vertex_cache, x_ = i.__wbg_get_meshoptconfig_preserve_uv_seams, y_ = i.__wbg_set_meshoptconfig_preserve_uv_seams, I_ = i.__wbg_get_meshoptconfig_preserve_boundaries, z_ = i.__wbg_set_meshoptconfig_preserve_boundaries, k_ = i.pixie_optimize_auto, $_ = i.pixie_optimize_image, j_ = i.pixie_optimize_mesh, F_ = i.pixie_get_memory_target_mb, A_ = i.pixie_reset_performance_stats, O_ = i.run_wasm_benchmarks, T_ = i.init, L_ = i.optimize_image, W_ = i.optimize_mesh, q_ = i.optimize_auto, E_ = i.version, B_ = i.build_timestamp, C_ = i.detect_format, S_ = i.get_performance_metrics, M_ = i.check_performance_compliance, R_ = i.optimize_png, G_ = i.optimize_jpeg, U_ = i.optimize_webp, D_ = i.optimize_gif, V_ = i.optimize_ico, H_ = i.optimize_tga, N_ = i.is_webp, Y_ = i.is_gif, K_ = i.is_ico, P_ = i.is_tga, Q_ = i.convert_to_webp, J_ = i.convert_to_png, X_ = i.convert_to_jpeg, Z_ = i.convert_to_bmp, te = i.convert_to_gif, _e = i.convert_to_ico, ee = i.convert_to_tiff, ne = i.strip_tiff_metadata_simd, oe = i.convert_to_svg, re = i.convert_to_tga, ie = i.set_lossless_mode, se = i.set_preserve_metadata, ge = i.optimize_obj, ce = i.optimize_gltf, ae = i.optimize_stl, pe = i.optimize_fbx, fe = i.optimize_ply, be = i.is_obj, we = i.is_gltf, de = i.is_stl, me = i.is_fbx, ue = i.is_ply, le = i.pixie_get_performance_stats, he = i.pixie_check_performance_compliance, ve = i.reset_performance_stats, xe = i.wasm_malloc, ye = i.wasm_free, Ie = i.wasm_get_memory_usage, ze = i.__wbindgen_export_0, ke = i.__wbindgen_export_1, $e = i.__wbindgen_export_2, je = i.__wbindgen_export_3, Fe = i.__wbindgen_add_to_stack_pointer, q = i.__wbindgen_start, Ae = Object.freeze(Object.defineProperty({
        __proto__: null,
        __wbg_get_imageoptconfig_compression_level: Nt,
        __wbg_get_imageoptconfig_enable_simd: Vt,
        __wbg_get_imageoptconfig_fast_mode: Kt,
        __wbg_get_imageoptconfig_lossless: qt,
        __wbg_get_imageoptconfig_max_colors: Rt,
        __wbg_get_imageoptconfig_max_height: t_,
        __wbg_get_imageoptconfig_max_width: Xt,
        __wbg_get_imageoptconfig_optimize_colors: St,
        __wbg_get_imageoptconfig_preserve_alpha: Qt,
        __wbg_get_imageoptconfig_preserve_metadata: Bt,
        __wbg_get_imageoptconfig_quality: Lt,
        __wbg_get_imageoptconfig_target_reduction: e_,
        __wbg_get_imageoptconfig_use_c_hotspots: Ut,
        __wbg_get_meshoptconfig_generate_normals: u_,
        __wbg_get_meshoptconfig_optimize_vertex_cache: h_,
        __wbg_get_meshoptconfig_preserve_boundaries: I_,
        __wbg_get_meshoptconfig_preserve_topology: s_,
        __wbg_get_meshoptconfig_preserve_uv_seams: x_,
        __wbg_get_meshoptconfig_simplification_algorithm: b_,
        __wbg_get_meshoptconfig_target_ratio: r_,
        __wbg_get_meshoptconfig_use_c_hotspots: d_,
        __wbg_get_meshoptconfig_vertex_tolerance: p_,
        __wbg_get_meshoptconfig_weld_vertices: c_,
        __wbg_imageoptconfig_free: Tt,
        __wbg_meshoptconfig_free: o_,
        __wbg_pixieconfig_free: xt,
        __wbg_set_imageoptconfig_compression_level: Yt,
        __wbg_set_imageoptconfig_enable_simd: Ht,
        __wbg_set_imageoptconfig_fast_mode: Pt,
        __wbg_set_imageoptconfig_lossless: Et,
        __wbg_set_imageoptconfig_max_colors: Gt,
        __wbg_set_imageoptconfig_max_height: __,
        __wbg_set_imageoptconfig_max_width: Zt,
        __wbg_set_imageoptconfig_optimize_colors: Mt,
        __wbg_set_imageoptconfig_preserve_alpha: Jt,
        __wbg_set_imageoptconfig_preserve_metadata: Ct,
        __wbg_set_imageoptconfig_quality: Wt,
        __wbg_set_imageoptconfig_target_reduction: n_,
        __wbg_set_imageoptconfig_use_c_hotspots: Dt,
        __wbg_set_meshoptconfig_generate_normals: l_,
        __wbg_set_meshoptconfig_optimize_vertex_cache: v_,
        __wbg_set_meshoptconfig_preserve_boundaries: z_,
        __wbg_set_meshoptconfig_preserve_topology: g_,
        __wbg_set_meshoptconfig_preserve_uv_seams: y_,
        __wbg_set_meshoptconfig_simplification_algorithm: w_,
        __wbg_set_meshoptconfig_target_ratio: i_,
        __wbg_set_meshoptconfig_use_c_hotspots: m_,
        __wbg_set_meshoptconfig_vertex_tolerance: f_,
        __wbg_set_meshoptconfig_weld_vertices: a_,
        __wbindgen_add_to_stack_pointer: Fe,
        __wbindgen_export_0: ze,
        __wbindgen_export_1: ke,
        __wbindgen_export_2: $e,
        __wbindgen_export_3: je,
        __wbindgen_start: q,
        build_timestamp: B_,
        check_performance_compliance: M_,
        convert_to_bmp: Z_,
        convert_to_gif: te,
        convert_to_ico: _e,
        convert_to_jpeg: X_,
        convert_to_png: J_,
        convert_to_svg: oe,
        convert_to_tga: re,
        convert_to_tiff: ee,
        convert_to_webp: Q_,
        detect_format: C_,
        get_performance_metrics: S_,
        init: T_,
        is_fbx: me,
        is_gif: Y_,
        is_gltf: we,
        is_ico: K_,
        is_obj: be,
        is_ply: ue,
        is_stl: de,
        is_tga: P_,
        is_webp: N_,
        memory: vt,
        optimize_auto: q_,
        optimize_fbx: pe,
        optimize_gif: D_,
        optimize_gltf: ce,
        optimize_ico: V_,
        optimize_image: L_,
        optimize_jpeg: G_,
        optimize_mesh: W_,
        optimize_obj: ge,
        optimize_ply: fe,
        optimize_png: R_,
        optimize_stl: ae,
        optimize_tga: H_,
        optimize_webp: U_,
        pixie_check_performance_compliance: he,
        pixie_get_memory_target_mb: F_,
        pixie_get_performance_stats: le,
        pixie_optimize_auto: k_,
        pixie_optimize_image: $_,
        pixie_optimize_mesh: j_,
        pixie_reset_performance_stats: A_,
        pixieconfig_enable_threading: jt,
        pixieconfig_new: yt,
        pixieconfig_quality: kt,
        pixieconfig_set_enable_threading: Ft,
        pixieconfig_set_quality: $t,
        pixieconfig_set_use_c_hotspots: zt,
        pixieconfig_to_image_config: At,
        pixieconfig_to_mesh_config: Ot,
        pixieconfig_use_c_hotspots: It,
        reset_performance_stats: ve,
        run_wasm_benchmarks: O_,
        set_lossless_mode: ie,
        set_preserve_metadata: se,
        strip_tiff_metadata_simd: ne,
        version: E_,
        wasm_free: ye,
        wasm_get_memory_usage: Ie,
        wasm_malloc: xe
    }, Symbol.toStringTag, {
        value: "Module"
    }));
    C(Ae);
    q();
})();
export { jn as ColorSpace, j as ImageOptConfig, F as MeshOptConfig, An as PixieConfig, Fn as SimplificationAlgorithm, D as __wbg_call_672a4d21634d4a24, V as __wbg_error_7534b8e9a36f1ab4, H as __wbg_instanceof_Window_def73ea0955fc569, N as __wbg_log_0cc1b7768397bcfe, Y as __wbg_log_8a185c59294f9f1f, K as __wbg_log_c222819a41e063d3, P as __wbg_log_cb9e190acc5753fb, Q as __wbg_mark_7438147ce31e9d4b, J as __wbg_measure_fb7825c11612c823, X as __wbg_new_405e22f390576ce2, Z as __wbg_new_78feb108b6472713, tt as __wbg_new_8a6f238a6ece86ea, _t as __wbg_newnoargs_105ed471475aaf50, et as __wbg_now_7f43f6c42e10de17, nt as __wbg_now_d18023d54d4e5500, ot as __wbg_performance_c185c0cdc2766575, rt as __wbg_set_37837023f3d740e8, it as __wbg_set_3f1d0b984ed272ed, C as __wbg_set_wasm, st as __wbg_stack_0ed75d68575b0f3c, gt as __wbg_static_accessor_GLOBAL_88a902d13a557d07, ct as __wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0, at as __wbg_static_accessor_SELF_37c5d418e4bf5819, pt as __wbg_static_accessor_WINDOW_5de37043a91a9c40, ft as __wbindgen_bigint_from_u64, bt as __wbindgen_error_new, wt as __wbindgen_is_undefined, dt as __wbindgen_number_new, mt as __wbindgen_object_clone_ref, ut as __wbindgen_object_drop_ref, lt as __wbindgen_string_new, ht as __wbindgen_throw, De as build_timestamp, Ye as check_performance_compliance, gn as convert_to_bmp, cn as convert_to_gif, an as convert_to_ico, sn as convert_to_jpeg, rn as convert_to_png, bn as convert_to_svg, wn as convert_to_tga, pn as convert_to_tiff, on as convert_to_webp, Ve as detect_format, He as get_performance_metrics, Se as init, kn as is_fbx, _n as is_gif, In as is_gltf, en as is_ico, yn as is_obj, $n as is_ply, zn as is_stl, nn as is_tga, tn as is_webp, Ge as optimize_auto, vn as optimize_fbx, Je as optimize_gif, ln as optimize_gltf, Xe as optimize_ico, Me as optimize_image, Pe as optimize_jpeg, Re as optimize_mesh, un as optimize_obj, xn as optimize_ply, Ke as optimize_png, hn as optimize_stl, Ze as optimize_tga, Qe as optimize_webp, Be as pixie_check_performance_compliance, We as pixie_get_memory_target_mb, qe as pixie_get_performance_stats, Oe as pixie_optimize_auto, Te as pixie_optimize_image, Le as pixie_optimize_mesh, Ee as pixie_reset_performance_stats, Ne as reset_performance_stats, Ce as run_wasm_benchmarks, dn as set_lossless_mode, mn as set_preserve_metadata, fn as strip_tiff_metadata_simd, Ue as version, __tla };
