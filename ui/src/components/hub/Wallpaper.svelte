<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { WallpaperSettings } from './hub.types';
  import { WALLPAPER_PRESETS } from './hubStore';

  let { settings }: { settings: WallpaperSettings } = $props();

  let canvas: HTMLCanvasElement | null = $state(null);
  let animId: number = 0;

  let activePreset = $derived(
    WALLPAPER_PRESETS.find((p) => p.id === settings.activeId) || WALLPAPER_PRESETS[0],
  );

  interface Particle {
    x: number;
    y: number;
    size: number;
    speedX: number;
    speedY: number;
    rotation: number;
    rotationSpeed: number;
    opacity: number;
    color: string;
  }

  let particles: Particle[] = [];

  function initParticles(width: number, height: number, type: string, count: number) {
    particles = [];
    for (let i = 0; i < count; i++) {
      if (type === 'sakura') {
        particles.push({
          x: Math.random() * width,
          y: Math.random() * height,
          size: 8 + Math.random() * 10,
          speedX: 0.5 + Math.random() * 1.5,
          speedY: 1.0 + Math.random() * 2.0,
          rotation: Math.random() * Math.PI * 2,
          rotationSpeed: (Math.random() - 0.5) * 0.05,
          opacity: 0.4 + Math.random() * 0.5,
          color: Math.random() > 0.3 ? '#ffb7c5' : '#ff9ebb',
        });
      } else if (type === 'cyberpunk') {
        particles.push({
          x: Math.random() * width,
          y: Math.random() * height,
          size: 2 + Math.random() * 4,
          speedX: (Math.random() - 0.5) * 0.5,
          speedY: -(0.5 + Math.random() * 1.5),
          rotation: 0,
          rotationSpeed: 0,
          opacity: 0.3 + Math.random() * 0.6,
          color: Math.random() > 0.5 ? '#00e5ff' : '#ff007f',
        });
      } else if (type === 'retrowave') {
        particles.push({
          x: Math.random() * width,
          y: Math.random() * height,
          size: 2 + Math.random() * 5,
          speedX: (Math.random() - 0.5) * 0.8,
          speedY: -(0.8 + Math.random() * 2.0),
          rotation: 0,
          rotationSpeed: 0,
          opacity: 0.4 + Math.random() * 0.5,
          color: Math.random() > 0.4 ? '#ff7700' : '#ff007f',
        });
      } else if (type === 'pixelart') {
        particles.push({
          x: Math.floor(Math.random() * (width / 4)) * 4,
          y: Math.floor(Math.random() * (height / 4)) * 4,
          size: 4,
          speedX: 0,
          speedY: -(0.5 + Math.random() * 1.0),
          rotation: 0,
          rotationSpeed: 0,
          opacity: 0.3 + Math.random() * 0.7,
          color: Math.random() > 0.5 ? '#7000ff' : '#00e5ff',
        });
      } else if (type === 'lofi-room' || type === 'lofi') {
        particles.push({
          x: Math.random() * width,
          y: Math.random() * height,
          size: 2 + Math.random() * 4,
          speedX: (Math.random() - 0.5) * 0.4,
          speedY: (Math.random() - 0.5) * 0.4,
          rotation: 0,
          rotationSpeed: 0,
          opacity: 0.3 + Math.random() * 0.6,
          color: Math.random() > 0.4 ? '#ffc078' : '#ffa94d',
        });
      } else {
        // Star / dust motes
        particles.push({
          x: Math.random() * width,
          y: Math.random() * height,
          size: 1 + Math.random() * 3,
          speedX: (Math.random() - 0.5) * 0.3,
          speedY: (Math.random() - 0.5) * 0.3,
          rotation: 0,
          rotationSpeed: 0,
          opacity: 0.2 + Math.random() * 0.7,
          color: '#ffffff',
        });
      }
    }
  }

  function startAnimation() {
    if (!canvas || !settings.particlesEnabled) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let width = (canvas.width = window.innerWidth);
    let height = (canvas.height = window.innerHeight);

    initParticles(width, height, activePreset.type, settings.particleDensity);

    const render = () => {
      ctx.clearRect(0, 0, width, height);

      for (const p of particles) {
        p.x += p.speedX;
        p.y += p.speedY;
        p.rotation += p.rotationSpeed;

        if (p.y > height + 20) {
          p.y = -20;
          p.x = Math.random() * width;
        } else if (p.y < -20) {
          p.y = height + 20;
          p.x = Math.random() * width;
        }

        if (p.x > width + 20) p.x = -20;
        else if (p.x < -20) p.x = width + 20;

        ctx.save();
        ctx.translate(p.x, p.y);
        ctx.rotate(p.rotation);
        ctx.globalAlpha = p.opacity;
        ctx.fillStyle = p.color;

        if (activePreset.type === 'sakura') {
          // Draw sakura petal (curved oval petal)
          ctx.beginPath();
          ctx.ellipse(0, 0, p.size * 0.6, p.size, 0, 0, Math.PI * 2);
          ctx.fill();
        } else if (activePreset.type === 'cyberpunk' || activePreset.type === 'pixelart') {
          // Draw neon diamond / pixel square
          ctx.beginPath();
          ctx.rect(-p.size / 2, -p.size / 2, p.size, p.size);
          ctx.shadowBlur = 6;
          ctx.shadowColor = p.color;
          ctx.fill();
        } else {
          // Draw soft glowing circle
          ctx.beginPath();
          ctx.arc(0, 0, p.size / 2, 0, Math.PI * 2);
          ctx.fill();
        }

        ctx.restore();
      }

      animId = requestAnimationFrame(render);
    };

    render();
  }

  function stopAnimation() {
    if (animId) {
      cancelAnimationFrame(animId);
      animId = 0;
    }
  }

  function handleResize() {
    if (canvas) {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      if (settings.particlesEnabled) {
        initParticles(canvas.width, canvas.height, activePreset.type, settings.particleDensity);
      }
    }
  }

  onMount(() => {
    window.addEventListener('resize', handleResize);
    if (settings.particlesEnabled) {
      startAnimation();
    }
  });

  onDestroy(() => {
    window.removeEventListener('resize', handleResize);
    stopAnimation();
  });

  $effect(() => {
    // React to preset or toggle changes
    stopAnimation();
    if (settings.particlesEnabled && canvas) {
      startAnimation();
    }
  });
</script>

<div
  class="wallpaper-container"
  data-testid="wallpaper-container"
  style="
    background: {settings.customUrl
    ? `url(${settings.customUrl}) center/cover no-repeat`
    : activePreset.gradient || activePreset.previewColor};
    filter: blur({settings.blur}px) brightness({settings.brightness});
    opacity: {settings.opacity};
  "
>
  {#if settings.particlesEnabled}
    <canvas bind:this={canvas} class="particle-canvas" data-testid="particle-canvas"></canvas>
  {/if}
</div>

<style>
  .wallpaper-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 0;
    pointer-events: none;
    transition:
      filter 0.3s ease,
      background 0.5s ease,
      opacity 0.3s ease;
    overflow: hidden;
  }

  .particle-canvas {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
</style>
