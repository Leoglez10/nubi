const tabs = [...document.querySelectorAll('[role="tab"]')];

function selectTab(selected) {
  for (const tab of tabs) {
    const active = tab === selected;
    const panel = document.getElementById(tab.getAttribute('aria-controls'));
    tab.setAttribute('aria-selected', String(active));
    tab.tabIndex = active ? 0 : -1;
    panel.hidden = !active;
    panel.tabIndex = active ? 0 : -1;
  }
}

tabs.forEach((tab, index) => {
  tab.addEventListener('click', () => selectTab(tab));
  tab.addEventListener('keydown', event => {
    const nextIndex = {
      ArrowRight: (index + 1) % tabs.length,
      ArrowLeft: (index + tabs.length - 1) % tabs.length,
      Home: 0,
      End: tabs.length - 1,
    }[event.key];
    if (nextIndex === undefined) return;
    event.preventDefault();
    selectTab(tabs[nextIndex]);
    tabs[nextIndex].focus();
  });
});

const flight = document.querySelector('.flight');
const stops = [...document.querySelectorAll('[data-stop]')];
const links = [...flight.querySelectorAll('a')];

if ('IntersectionObserver' in window) {
  const cloud = document.createElement('span');
  cloud.className = 'flight-cloud';
  const mascot = new Image(120, 96);
  mascot.src = 'nubi.svg';
  mascot.alt = '';
  cloud.append(mascot);
  flight.querySelector('.flight-window').append(cloud);

  // Select the stop nearest the reading line, including after anchor jumps.
  const fullMotion = matchMedia('(width > 60rem) and (prefers-reduced-motion: no-preference)');
  let observer;
  let activeStop;
  let bank;
  fullMotion.addEventListener('change', () => bank?.cancel());

  function markStop(readingLine) {
    const next = stops.findIndex(stop => stop.getBoundingClientRect().bottom > readingLine);
    const index = next === -1 ? stops.length - 1 : next;
    if (index === activeStop) return;
    if (activeStop !== undefined && fullMotion.matches && typeof mascot.animate === 'function') {
      const tilt = index > activeStop ? 10 : -10;
      bank?.cancel();
      bank = mascot.animate([
        { translate: '0px', rotate: '0deg' },
        { translate: '1.1rem', rotate: `${tilt}deg`, offset: .42 },
        { translate: '.35rem', rotate: `${-tilt / 2}deg`, offset: .78 },
        { translate: '0px', rotate: '0deg' },
      ], { duration: 640, easing: 'ease-in-out' });
    }
    activeStop = index;
    flight.style.setProperty('--stop', index);
    links.forEach((link, item) => {
      if (item === index) link.setAttribute('aria-current', 'location');
      else link.removeAttribute('aria-current');
    });
  }

  function observeStops() {
    observer?.disconnect();
    const height = document.documentElement.clientHeight;
    const line = Math.round(height * .35);
    observer = new IntersectionObserver(() => markStop(line), {
      rootMargin: `-${line}px 0px -${height - line - 1}px 0px`,
    });
    stops.forEach(stop => observer.observe(stop));
  }

  observeStops();
  window.addEventListener('resize', observeStops);
}

const platform = navigator.userAgent;
const os = /Mac/.test(platform) ? 'mac'
  : /Win/.test(platform) ? 'win'
  : /Linux/.test(platform) && !/Android/.test(platform) ? 'linux'
  : null;
if (os) document.querySelector(`.os[data-os="${os}"]`)?.classList.add('mine');

const copyStatus = document.querySelector('#copy-status');
document.querySelectorAll('.copy').forEach(button => {
  const source = button.previousElementSibling;
  const label = button.textContent;
  let reset;
  button.hidden = false;

  button.addEventListener('click', async () => {
    clearTimeout(reset);
    try {
      await navigator.clipboard.writeText(source.textContent);
      button.textContent = 'Copiado';
      copyStatus.textContent = 'Comando copiado.';
    } catch {
      button.textContent = 'Reintentar';
      copyStatus.textContent = 'No se pudo copiar. Selecciona el comando para copiarlo manualmente.';
    }
    reset = setTimeout(() => {
      button.textContent = label;
      copyStatus.textContent = '';
    }, 2200);
  });
});
