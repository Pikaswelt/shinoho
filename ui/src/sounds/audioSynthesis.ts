import type { SoundEvent, SoundPackId } from './types';

let audioCtx: AudioContext | null = null;

export function getAudioContext(): AudioContext | null {
  if (audioCtx) {
    if (audioCtx.state === 'suspended') {
      audioCtx.resume().catch(() => {});
    }
    return audioCtx;
  }

  if (typeof window === 'undefined') return null;
  const AudioContextClass =
    window.AudioContext ||
    (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AudioContextClass) return null;

  try {
    audioCtx = new AudioContextClass();
  } catch {
    return null;
  }

  if (audioCtx.state === 'suspended') {
    audioCtx.resume().catch(() => {});
  }

  return audioCtx;
}

export function setCustomAudioContext(ctx: AudioContext | null): void {
  audioCtx = ctx;
}

function playTone(
  ctx: AudioContext,
  opts: {
    freq: number;
    endFreq?: number;
    type?: OscillatorType;
    duration: number;
    volume: number;
    attack?: number;
    delay?: number;
  },
) {
  const now = ctx.currentTime + (opts.delay || 0);
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();

  osc.type = opts.type || 'sine';
  osc.frequency.setValueAtTime(opts.freq, now);
  if (opts.endFreq) {
    osc.frequency.exponentialRampToValueAtTime(Math.max(10, opts.endFreq), now + opts.duration);
  }

  const peakVol = Math.max(0.0001, opts.volume);
  const attack = opts.attack || 0.005;

  gain.gain.setValueAtTime(0.0001, now);
  gain.gain.linearRampToValueAtTime(peakVol, now + attack);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + opts.duration);

  osc.connect(gain);
  gain.connect(ctx.destination);

  osc.start(now);
  osc.stop(now + opts.duration + 0.05);
}

export function synthesizeSound(packId: SoundPackId, event: SoundEvent, baseVolume: number): void {
  const ctx = getAudioContext();
  if (!ctx) return;

  const vol = Math.max(0, Math.min(1, baseVolume));
  if (vol <= 0.001) return;

  switch (packId) {
    case 'kawaii':
      playKawaiiSound(ctx, event, vol);
      break;
    case 'cyberpunk':
      playCyberpunkSound(ctx, event, vol);
      break;
    case 'gaming':
      playGamingSound(ctx, event, vol);
      break;
    case 'mecha':
      playMechaSound(ctx, event, vol);
      break;
    case 'subtle':
    default:
      playSubtleSound(ctx, event, vol);
      break;
  }
}

function playSubtleSound(ctx: AudioContext, event: SoundEvent, vol: number) {
  switch (event) {
    case 'tab.open':
      playTone(ctx, { freq: 480, endFreq: 640, type: 'sine', duration: 0.08, volume: vol * 0.7 });
      break;
    case 'tab.close':
      playTone(ctx, { freq: 440, endFreq: 260, type: 'sine', duration: 0.07, volume: vol * 0.6 });
      break;
    case 'tab.switch':
      playTone(ctx, { freq: 520, type: 'sine', duration: 0.04, volume: vol * 0.5 });
      break;
    case 'download.start':
      playTone(ctx, { freq: 440, type: 'sine', duration: 0.06, volume: vol * 0.7 });
      playTone(ctx, { freq: 587, type: 'sine', duration: 0.09, volume: vol * 0.7, delay: 0.06 });
      break;
    case 'download.done':
      playTone(ctx, { freq: 523, type: 'sine', duration: 0.07, volume: vol * 0.8 });
      playTone(ctx, { freq: 659, type: 'sine', duration: 0.07, volume: vol * 0.8, delay: 0.06 });
      playTone(ctx, { freq: 784, type: 'sine', duration: 0.15, volume: vol * 0.85, delay: 0.12 });
      break;
    case 'click':
      playTone(ctx, { freq: 700, type: 'sine', duration: 0.025, volume: vol * 0.4 });
      break;
  }
}

function playKawaiiSound(ctx: AudioContext, event: SoundEvent, vol: number) {
  switch (event) {
    case 'tab.open':
      playTone(ctx, { freq: 880, endFreq: 1175, type: 'sine', duration: 0.11, volume: vol * 0.6 });
      playTone(ctx, {
        freq: 1760,
        endFreq: 2350,
        type: 'triangle',
        duration: 0.09,
        volume: vol * 0.25,
      });
      break;
    case 'tab.close':
      playTone(ctx, { freq: 784, endFreq: 523, type: 'sine', duration: 0.09, volume: vol * 0.5 });
      break;
    case 'tab.switch':
      playTone(ctx, { freq: 988, type: 'sine', duration: 0.05, volume: vol * 0.5 });
      break;
    case 'download.start':
      playTone(ctx, { freq: 659, type: 'sine', duration: 0.07, volume: vol * 0.6 });
      playTone(ctx, { freq: 880, type: 'sine', duration: 0.1, volume: vol * 0.65, delay: 0.06 });
      break;
    case 'download.done':
      playTone(ctx, { freq: 587, type: 'sine', duration: 0.08, volume: vol * 0.7 });
      playTone(ctx, { freq: 740, type: 'sine', duration: 0.08, volume: vol * 0.75, delay: 0.06 });
      playTone(ctx, { freq: 880, type: 'sine', duration: 0.08, volume: vol * 0.8, delay: 0.12 });
      playTone(ctx, { freq: 1175, type: 'sine', duration: 0.18, volume: vol * 0.85, delay: 0.18 });
      break;
    case 'click':
      playTone(ctx, { freq: 1046, type: 'sine', duration: 0.03, volume: vol * 0.45 });
      break;
  }
}

function playCyberpunkSound(ctx: AudioContext, event: SoundEvent, vol: number) {
  switch (event) {
    case 'tab.open':
      playTone(ctx, {
        freq: 320,
        endFreq: 880,
        type: 'sawtooth',
        duration: 0.08,
        volume: vol * 0.45,
      });
      break;
    case 'tab.close':
      playTone(ctx, {
        freq: 440,
        endFreq: 140,
        type: 'sawtooth',
        duration: 0.07,
        volume: vol * 0.4,
      });
      break;
    case 'tab.switch':
      playTone(ctx, {
        freq: 620,
        endFreq: 850,
        type: 'sawtooth',
        duration: 0.035,
        volume: vol * 0.35,
      });
      break;
    case 'download.start':
      playTone(ctx, { freq: 380, endFreq: 760, type: 'square', duration: 0.07, volume: vol * 0.4 });
      break;
    case 'download.done':
      playTone(ctx, { freq: 440, type: 'square', duration: 0.05, volume: vol * 0.45 });
      playTone(ctx, { freq: 659, type: 'square', duration: 0.05, volume: vol * 0.5, delay: 0.04 });
      playTone(ctx, { freq: 880, type: 'square', duration: 0.12, volume: vol * 0.55, delay: 0.08 });
      break;
    case 'click':
      playTone(ctx, { freq: 950, type: 'square', duration: 0.02, volume: vol * 0.3 });
      break;
  }
}

function playGamingSound(ctx: AudioContext, event: SoundEvent, vol: number) {
  switch (event) {
    case 'tab.open':
      playTone(ctx, { freq: 260, type: 'square', duration: 0.04, volume: vol * 0.45 });
      playTone(ctx, { freq: 520, type: 'square', duration: 0.07, volume: vol * 0.5, delay: 0.035 });
      break;
    case 'tab.close':
      playTone(ctx, { freq: 390, type: 'square', duration: 0.04, volume: vol * 0.45 });
      playTone(ctx, { freq: 195, type: 'square', duration: 0.07, volume: vol * 0.4, delay: 0.035 });
      break;
    case 'tab.switch':
      playTone(ctx, { freq: 660, type: 'square', duration: 0.04, volume: vol * 0.4 });
      break;
    case 'download.start':
      playTone(ctx, {
        freq: 330,
        endFreq: 660,
        type: 'square',
        duration: 0.08,
        volume: vol * 0.45,
      });
      break;
    case 'download.done':
      playTone(ctx, { freq: 330, type: 'square', duration: 0.05, volume: vol * 0.5 });
      playTone(ctx, { freq: 440, type: 'square', duration: 0.05, volume: vol * 0.5, delay: 0.04 });
      playTone(ctx, { freq: 554, type: 'square', duration: 0.05, volume: vol * 0.55, delay: 0.08 });
      playTone(ctx, { freq: 660, type: 'square', duration: 0.14, volume: vol * 0.6, delay: 0.12 });
      break;
    case 'click':
      playTone(ctx, { freq: 780, type: 'square', duration: 0.025, volume: vol * 0.35 });
      break;
  }
}

function playMechaSound(ctx: AudioContext, event: SoundEvent, vol: number) {
  switch (event) {
    case 'tab.open':
      playTone(ctx, {
        freq: 220,
        endFreq: 380,
        type: 'triangle',
        duration: 0.09,
        volume: vol * 0.6,
      });
      playTone(ctx, { freq: 440, endFreq: 760, type: 'sine', duration: 0.07, volume: vol * 0.3 });
      break;
    case 'tab.close':
      playTone(ctx, {
        freq: 320,
        endFreq: 160,
        type: 'triangle',
        duration: 0.08,
        volume: vol * 0.5,
      });
      break;
    case 'tab.switch':
      playTone(ctx, { freq: 580, type: 'triangle', duration: 0.04, volume: vol * 0.5 });
      break;
    case 'download.start':
      playTone(ctx, {
        freq: 200,
        endFreq: 400,
        type: 'triangle',
        duration: 0.08,
        volume: vol * 0.55,
      });
      break;
    case 'download.done':
      playTone(ctx, { freq: 392, type: 'triangle', duration: 0.06, volume: vol * 0.6 });
      playTone(ctx, {
        freq: 784,
        type: 'triangle',
        duration: 0.12,
        volume: vol * 0.65,
        delay: 0.05,
      });
      break;
    case 'click':
      playTone(ctx, { freq: 680, type: 'triangle', duration: 0.025, volume: vol * 0.4 });
      break;
  }
}
