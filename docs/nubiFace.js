// Nubi's living face: every feature is a number on a spring, drawn into an <svg viewBox="0 0 120 96">.
// The landing page uses a compiled copy: after editing, run `npm run face` in ui/ (writes docs/nubiFace.js).
// A pose is a flat list of numbers so any two faces can morph into each other:
//  0-11 mouth: left, upper-left ctrl, upper-right ctrl, right, lower-right ctrl, lower-left ctrl
// 12-19 brows: left (outer, inner), right (inner, outer)
//    20 brow opacity
// 21-24 lid bottom edge y: left outer, left inner, right inner, right outer (47 = fully open)
//    25 pupil radius
//    26 rage: red cheeks, anger mark, trembling
const line = (lx, ly, c1x, c1y, c2x, c2y, rx, ry) => [lx, ly, c1x, c1y, c2x, c2y, rx, ry, c2x, c2y, c1x, c1y];
const HIDDEN_BROWS = [44, 44, 55, 44, 69, 44, 80, 44, 0];
const OPEN = [47, 47, 47, 47];
const POSES = {
    neutral: [...line(54, 64, 59.3, 68.7, 64.7, 68.7, 70, 64), ...HIDDEN_BROWS, ...OPEN, 5, 0],
    happy: [...line(51, 62, 58.3, 69.3, 65.7, 69.3, 73, 62), ...HIDDEN_BROWS, ...OPEN, 5, 0],
    worried: [...line(54, 68, 59.3, 63.3, 64.7, 63.3, 70, 68), 44, 46, 55, 43, 69, 43, 80, 46, 1, ...OPEN, 5, 0],
    surprised: [57, 67, 57.5, 61, 66.5, 61, 67, 67, 66.5, 73, 57.5, 73, 44, 42, 55, 40, 69, 40, 80, 42, 1, ...OPEN, 3.8, 0],
    annoyed: [...line(55, 67, 60, 67, 64, 67, 69, 67), 43, 45, 55, 45.5, 69, 45.5, 81, 45, 1, 51, 51, 51, 51, 5, 0],
    angry: [52, 70, 57, 62.5, 67, 62.5, 72, 70, 67, 66, 57, 66, 42, 40, 56, 46.5, 68, 46.5, 82, 40, 1, 48, 52.5, 52.5, 48, 5, 1],
};
/** Each click in a row pokes Nubi one step closer to angry. */
const POKED = ['surprised', 'worried', 'annoyed', 'angry'];
const CALM_MS = 2200;
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
  <path data-part="brows" fill="none" stroke="${INK}" stroke-width="3" stroke-linecap="round" opacity="0"/>
  <path data-part="mouth" fill="${INK}" stroke="${INK}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
  <path data-part="vein" d="M-5-1.5q3.5 0 3.5-3.5M1.5-5q0 3.5 3.5 3.5M5 1.5q-3.5 0-3.5 3.5M-1.5 5q0-3.5-3.5-3.5"
    fill="none" stroke="#d92d20" stroke-width="2.2" stroke-linecap="round" opacity="0"/>
</g>`;
const f = (n) => n.toFixed(2);
/** Draws Nubi into `svg` and keeps it alive until `destroy`. With `still`, nothing moves on its own. */
export function nubiFace(svg, rest, still) {
    svg.innerHTML = SVG;
    const part = (name) => svg.querySelector(`[data-part="${name}"]`);
    const [body, rage, eyes, glints, lids, brows, mouth, vein] = ['body', 'rage-cheeks', 'eyes', 'glints', 'lids', 'brows', 'mouth', 'vein'].map(part);
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
    // The easter egg: poke Nubi and it gets grumpier until left alone.
    const onClick = () => {
        pokes = Math.min(pokes + 1, POKED.length);
        squash.v += 0.07;
        set(POKED[pokes - 1]);
        clearTimeout(calm);
        calm = setTimeout(() => { pokes = 0; set(rest); }, CALM_MS);
    };
    svg.addEventListener('click', onClick);
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
            if (!pokes)
                set(next);
        },
        destroy() {
            if (frame)
                cancelAnimationFrame(frame);
            clearTimeout(calm);
            clearTimeout(nextBlink);
            svg.removeEventListener('click', onClick);
            window.removeEventListener('pointermove', onPointer);
        },
    };
}
