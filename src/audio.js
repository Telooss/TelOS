// Moteur audio UI — Zéro latence (Web Audio API)
// Loi n°1 : La navigation ne doit jamais attendre, l'audio non plus.
// HTMLAudioElement (<audio>) est inadapté car il ajoute un délai au démarrage
// et ne gère pas la superposition de sons identiques lancés en rafale.

const audio = (() => {
  // Le contexte n'est créé qu'à l'initialisation pour respecter les
  // politiques "autoplay" des navigateurs (requiert une interaction).
  let ctx = null;
  const buffers = new Map();

  // Volume global discret pour l'UI (pas besoin de casser les oreilles).
  let masterGain = null;

  async function init() {
    if (ctx) return;
    const AudioContext = window.AudioContext || window.webkitAudioContext;
    ctx = new AudioContext();
    
    masterGain = ctx.createGain();
    masterGain.gain.value = 0.4; // 40% du volume max
    masterGain.connect(ctx.destination);

    // Charge les sons de manière asynchrone sans bloquer le rendu visuel.
    await load('move', 'audio/Minimal-move-sound.wav');
  }

  async function load(name, url) {
    try {
      const res = await fetch(url);
      const arrayBuffer = await res.arrayBuffer();
      const audioBuffer = await ctx.decodeAudioData(arrayBuffer);
      buffers.set(name, audioBuffer);
    } catch (e) {
      console.error(`[telOS/audio] Échec du chargement du son '${name}' (${url})`, e);
    }
  }

  function play(name) {
    if (!ctx || ctx.state === 'suspended') {
      // Les navigateurs peuvent suspendre l'audio s'il n'y a pas eu d'interaction
      if (ctx) ctx.resume();
      else return; // Pas encore initialisé
    }
    
    const buffer = buffers.get(name);
    if (!buffer) return;

    // Création d'une nouvelle source à la volée. C'est extrêmement léger
    // et ça permet au même son de se chevaucher si on scrolle vite.
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(masterGain);
    source.start(0);
  }

  // L'initialisation doit se faire sur la toute première interaction
  // (clavier, clic de souris, ou appui manette).
  const enable = () => {
    init();
    window.removeEventListener('keydown', enable);
    window.removeEventListener('mousedown', enable);
    window.removeEventListener('gamepadconnected', enable);
  };

  window.addEventListener('keydown', enable, { once: true });
  window.addEventListener('mousedown', enable, { once: true });
  window.addEventListener('gamepadconnected', enable, { once: true });

  return { play };
})();
