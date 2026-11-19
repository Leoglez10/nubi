// Nubi's living face: every feature is a number on a spring, drawn into an <svg viewBox="0 0 120 96">.
// The landing page uses a compiled copy: after editing, run `npm run face` in ui/ (writes docs/nubiFace.js).
// A pose is a flat list of numbers so any two faces can morph into each other:
//  0-11 mouth: left, upper-left ctrl, upper-right ctrl, right, lower-right ctrl, lower-left ctrl
// 12-19 brows: left (outer, inner), right (inner, outer)
//    20 brow opacity
// 21-24 lid bottom edge y: left outer, left inner, right inner, right outer (47 = fully open)
//    25 pupil radius
//    26 rage: red cheeks, anger mark, trembling
//    27 sleep: closed eyes and floating z's
const line = (lx, ly, c1x, c1y, c2x, c2y, rx, ry) => [lx, ly, c1x, c1y, c2x, c2y, rx, ry, c2x, c2y, c1x, c1y];
const HIDDEN_BROWS = [44, 44, 55, 44, 69, 44, 80, 44, 0];
const OPEN = [47, 47, 47, 47];
const POSES = {
    neutral: [...line(54, 64, 59.3, 68.7, 64.7, 68.7, 70, 64), ...HIDDEN_BROWS, ...OPEN, 5, 0, 0],
    happy: [...line(51, 62, 58.3, 69.3, 65.7, 69.3, 73, 62), ...HIDDEN_BROWS, ...OPEN, 5, 0, 0],
    worried: [...line(54, 68, 59.3, 63.3, 64.7, 63.3, 70, 68), 44, 46, 55, 43, 69, 43, 80, 46, 1, ...OPEN, 5, 0, 0],
    surprised: [57, 67, 57.5, 61, 66.5, 61, 67, 67, 66.5, 73, 57.5, 73, 44, 42, 55, 40, 69, 40, 80, 42, 1, ...OPEN, 3.8, 0, 0],
    annoyed: [...line(55, 67, 60, 67, 64, 67, 69, 67), 43, 45, 55, 45.5, 69, 45.5, 81, 45, 1, 51, 51, 51, 51, 5, 0, 0],
    angry: [52, 70, 57, 62.5, 67, 62.5, 72, 70, 67, 66, 57, 66, 42, 40, 56, 46.5, 68, 46.5, 82, 40, 1, 48, 52.5, 52.5, 48, 5, 1, 0],
    asleep: [...line(57, 66, 60, 67.5, 64, 67.5, 67, 66), ...HIDDEN_BROWS, 59.6, 59.6, 59.6, 59.6, 5, 0, 1],
};
/** Each click in a row pokes Nubi one step closer to angry. */
const POKED = ['surprised', 'worried', 'annoyed', 'angry'];
const CALM_MS = 2200;
/** Without pointer or keyboard activity for this long, Nubi dozes off. */
const IDLE_MS = 30000;
const INK = '#101828';
const SVG = `
<g data-part="body">
  <path d="M30 80h62a22 22 0 0 0 2-44 30 30 0 0 0-57-6A24 24 0 0 0 30 80z" fill="#fff" stroke="${INK}" stroke-width="3.5" stroke-linejoin="round"/>
  <circle cx="42" cy="64" r="4" fill="#fda29b"/><circle cx="82" cy="64" r="4" fill="#fda29b"/>
  <g data-part="rage-cheeks" opacity="0"><circle cx="42" cy="64" r="4.6" fill="#f04438"/><circle cx="82" cy="64" r="4.6" fill="#f04438"/></g>
  <g data-part="eyes">
    <circle data-part="pupil" cx="50" cy="54" r="5" fill="${INK}"/><circle data-part="pupil" cx="74" cy="54" r="5" fill="${INK}"/>
    <g data-part="glints" transform="translate(2 -2)"><circle cx="50" cy="54" r="1.6" fill="#fff"/><circle cx="74" cy="54" r="1.6" fill="#fff"/></g>
  </g>
  <path data-part="lids" fill="#fff"/>
  <path data-part="closed" d="M45 55q5 3.5 10 0M69 55q5 3.5 10 0" fill="none" stroke="${INK}" stroke-width="3" stroke-linecap="round" opacity="0"/>
  <path data-part="brows" fill="none" stroke="${INK}" stroke-width="3" stroke-linecap="round" opacity="0"/>
  <path data-part="mouth" fill="${INK}" stroke="${INK}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
  <g data-part="zzz" opacity="0" fill="${INK}" font-family="system-ui, sans-serif" font-weight="800">ZZZ</g>
  <path data-part="vein" d="M-5-1.5q3.5 0 3.5-3.5M1.5-5q0 3.5 3.5 3.5M5 1.5q-3.5 0-3.5 3.5M-1.5 5q0-3.5-3.5-3.5"
    fill="none" stroke="#d92d20" stroke-width="2.2" stroke-linecap="round" opacity="0"/>
</g>`;
const f = (n) => n.toFixed(2);
/** Draws Nubi into `svg` and keeps it alive until `destroy`. With `still`, nothing moves on its own. */
export function nubiFace(svg, rest, still) {
    // At rest Nubi breathes: a slow, slight stretch from its base (SVG-native, so no frame loop).
    // Each Nubi starts at a random point of the breath so several on screen don't move in step.
    const breath = still ? '' : `<animateTransform attributeName="transform" type="scale" values="1 1;1.012 1.03;1 1"
    dur="3.8s" begin="-${(Math.random() * 3.8).toFixed(2)}s" repeatCount="indefinite" calcMode="spline" keySplines=".45 0 .55 1;.45 0 .55 1"/>`;
    // Sleeping z's drift up and fade, one after another.
    const zs = [0, 1, 2].map((i) => still
        ? `<text x="${88 + i * 6}" y="${32 - i * 7}" font-size="${7 + i * 2}">z</text>`
        : `<text x="88" y="32" font-size="9" opacity="0">z<animateTransform attributeName="transform" type="translate"
        values="0 0;10 -18" dur="2.4s" begin="${i * 0.8}s" repeatCount="indefinite"/><animate attributeName="opacity"
        values="0;1;0" dur="2.4s" begin="${i * 0.8}s" repeatCount="indefinite"/></text>`).join('');
    svg.innerHTML = `<g transform="translate(60 80)"><g>${breath}<g transform="translate(-60 -80)">${SVG.replace('ZZZ', zs)}</g></g></g>`;
    const part = (name) => svg.querySelector(`[data-part="${name}"]`);
    const [body, rage, eyes, glints, lids, brows, mouth, vein, closed, zzz] = ['body', 'rage-cheeks', 'eyes', 'glints', 'lids', 'brows', 'mouth', 'vein', 'closed', 'zzz'].map(part);
    const pupils = svg.querySelectorAll('[data-part="pupil"]');
    let face = rest;
    let pokes = 0;
    const pose = [...POSES[face]];
    const speed = pose.map(() => 0);
    // Glints (x, y) and body squash also ride springs.
    const look = { x: 2, y: -2, vx: 0, vy: 0, tx: 2, ty: -2 };
    const squash = { x: 0, v: 0 };
    let blinkAt = -1;
    let frame = 0;
    let calm;
    let nextBlink;
    function draw(now) {
        const p = pose;
        mouth.setAttribute('d', `M${f(p[0])} ${f(p[1])}C${f(p[2])} ${f(p[3])} ${f(p[4])} ${f(p[5])} ${f(p[6])} ${f(p[7])}` +
            `C${f(p[8])} ${f(p[9])} ${f(p[10])} ${f(p[11])} ${f(p[0])} ${f(p[1])}Z`);
        brows.setAttribute('d', `M${f(p[12])} ${f(p[13])}L${f(p[14])} ${f(p[15])}M${f(p[16])} ${f(p[17])}L${f(p[18])} ${f(p[19])}`);
        brows.setAttribute('opacity', f(Math.min(1, Math.max(0, p[20]))));
        lids.setAttribute('d', `M43 36H57V${f(p[22])}L43 ${f(p[21])}ZM67 36H81V${f(p[24])}L67 ${f(p[23])}Z`);
        pupils.forEach((c) => c.setAttribute('r', f(p[25])));
        const r = Math.min(1, Math.max(0, p[26]));
        rage.setAttribute('opacity', f(r));
        vein.setAttribute('opacity', f(r));
        const z = Math.min(1, Math.max(0, p[27]));
        closed.setAttribute('opacity', f(z));
        zzz.setAttribute('opacity', f(z));
        vein.setAttribute('transform', `translate(95 30) scale(${f(0.4 + 0.6 * r)}) rotate(${f(r * 12)})`);
        glints.setAttribute('transform', `translate(${f(look.x)} ${f(look.y)})`);
        // A blink is a quick close-and-open of the eyes (pupils and glints) around their center line.
        const b = blinkAt < 0 ? 0 : Math.sin(Math.PI * Math.min(1, (now - blinkAt) / 170));
        eyes.setAttribute('transform', `translate(0 54) scale(1 ${f(1 - 0.9 * b)}) translate(0 -54)`);
        const tremble = r > 0.5 ? Math.sin(now / 11) * 0.9 * r : 0;
        const s = squash.x;
        body.setAttribute('transform', `translate(${f(60 + tremble)} 80) scale(${f(1 + s)} ${f(1 - s)}) translate(-60 -80)`);
    }
    function step(now) {
        frame = 0;
        const target = POSES[face];
        let moving = false;
        for (let i = 0; i < pose.length; i++) {
            speed[i] = (speed[i] + (target[i] - pose[i]) * 0.14) * 0.72;
            pose[i] += speed[i];
            if (Math.abs(speed[i]) > 0.002 || Math.abs(target[i] - pose[i]) > 0.01)
                moving = true;
        }
        look.vx = (look.vx + (look.tx - look.x) * 0.12) * 0.7;
        look.vy = (look.vy + (look.ty - look.y) * 0.12) * 0.7;
        look.x += look.vx;
        look.y += look.vy;
        squash.v = (squash.v - squash.x * 0.22) * 0.78;
        squash.x += squash.v;
        if (blinkAt >= 0 && now - blinkAt > 170)
            blinkAt = -1;
        moving ||= Math.abs(look.tx - look.x) + Math.abs(look.ty - look.y) > 0.01 || Math.abs(squash.x) + Math.abs(squash.v) > 0.0005 ||
            blinkAt >= 0 || pose[26] > 0.5;
        draw(now);
        if (moving)
            frame = requestAnimationFrame(step);
    }
    const wake = () => {
        if (still)
            return draw(performance.now());
        if (!frame)
            frame = requestAnimationFrame(step);
    };
    function set(next) {
        face = next;
        if (still)
            pose.splice(0, pose.length, ...POSES[face]);
        wake();
    }
    function scheduleBlink() {
        // Every few seconds, now and then twice in a row.
        nextBlink = setTimeout(() => {
            if (asleep)
                return scheduleBlink();
            blinkAt = performance.now();
            wake();
            if (Math.random() < 0.2)
                setTimeout(() => { blinkAt = performance.now(); wake(); }, 260);
            scheduleBlink();
        }, 2500 + Math.random() * 4000);
    }
    const onPointer = (e) => {
        const box = svg.getBoundingClientRect();
        if (!box.width)
            return;
        const dx = e.clientX - (box.left + box.width * 0.52);
        const dy = e.clientY - (box.top + box.height * 0.56);
        // Up to 3 units off center keeps the glint (r 1.6) inside the pupil (r 5).
        const k = 3 / Math.max(Math.hypot(dx, dy), 60);
        look.tx = dx * k;
        look.ty = dy * k;
        wake();
    };
    // Idle for a while, Nubi falls asleep; any pointer or key activity wakes it (blinking awake).
    let asleep = false;
    let idle;
    const doze = () => {
        asleep = true;
        pokes = 0;
        clearTimeout(calm);
        set('asleep');
    };
    const stir = () => {
        clearTimeout(idle);
        idle = setTimeout(doze, IDLE_MS);
        if (!asleep)
            return;
        asleep = false;
        set(rest);
        blinkAt = performance.now();
    };
    // The easter egg: poke Nubi and it gets grumpier until left alone.
    const onClick = () => {
        pokes = Math.min(pokes + 1, POKED.length);
        squash.v += 0.07;
        set(POKED[pokes - 1]);
        clearTimeout(calm);
        calm = setTimeout(() => { pokes = 0; set(rest); }, CALM_MS);
    };
    svg.addEventListener('click', onClick);
    const ACTIVITY = ['pointermove', 'pointerdown', 'keydown'];
    ACTIVITY.forEach((type) => window.addEventListener(type, stir, { passive: true }));
    stir();
    if (!still && typeof requestAnimationFrame === 'function') {
        window.addEventListener('pointermove', onPointer, { passive: true });
        scheduleBlink();
    }
    else {
        still = true;
    }
    draw(0);
    return {
        /** Changes the face Nubi rests on, e.g. when a screen's mood changes. */
        rest(next) {
            rest = next;
            if (!pokes && !asleep)
                set(next);
        },
        destroy() {
            if (frame)
                cancelAnimationFrame(frame);
            clearTimeout(calm);
            clearTimeout(nextBlink);
            clearTimeout(idle);
            ACTIVITY.forEach((type) => window.removeEventListener(type, stir));
            svg.removeEventListener('click', onClick);
            window.removeEventListener('pointermove', onPointer);
        },
    };
}
