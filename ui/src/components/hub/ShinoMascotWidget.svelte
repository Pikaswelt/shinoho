<script lang="ts">
  import { tHub } from './i18n';

  let clickCount = $state(0);
  let reaction = $state<string | null>(null);

  function getGreetingKey(): string {
    const hour = new Date().getHours();
    if (hour >= 5 && hour < 12) return 'hub.mascot.greetingMorning';
    if (hour >= 12 && hour < 18) return 'hub.mascot.greetingAfternoon';
    if (hour >= 18 && hour < 23) return 'hub.mascot.greetingEvening';
    return 'hub.mascot.greetingNight';
  }

  const extraReactions = [
    '(≧◡≦) ♡ Gambatte!',
    '(ง •_•)ง Zeit für epische Abenteuer!',
    '🌸 *streut Sakura-Blüten*',
    '(¬‿¬) Psst, schon alle neuen Anime-Folgen gesehen?',
    '✨ Sugoi!',
  ];

  function handleClick() {
    clickCount++;
    reaction = extraReactions[clickCount % extraReactions.length];
  }
</script>

<div class="mascot-widget" data-testid="shino-mascot-widget">
  <button type="button" class="mascot-avatar-btn" onclick={handleClick} title="Shino-chan">
    <div class="chibi-avatar">
      <span class="avatar-face">👧🏻🌸</span>
      <span class="avatar-glow"></span>
    </div>
  </button>

  <div class="speech-bubble" data-testid="mascot-speech-bubble">
    <span class="speaker-name">Shino-chan:</span>
    <p class="bubble-text">
      {reaction || tHub(getGreetingKey())}
    </p>
  </div>
</div>

<style>
  .mascot-widget {
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(24, 24, 34, 0.65);
    backdrop-filter: blur(12px);
    border: 1px solid var(--color-border, #2a2a3c);
    border-radius: var(--radius-lg, 12px);
    padding: 10px 16px;
    max-width: 520px;
    margin: 0 auto;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }

  .mascot-avatar-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    outline: none;
  }

  .chibi-avatar {
    position: relative;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: linear-gradient(135deg, rgba(255, 71, 133, 0.3), rgba(0, 229, 255, 0.3));
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--color-accent, #ff4785);
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  }

  .mascot-avatar-btn:hover .chibi-avatar {
    transform: scale(1.1) rotate(6deg);
  }

  .avatar-face {
    font-size: 24px;
  }

  .speech-bubble {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .speaker-name {
    font-size: 11px;
    font-weight: 700;
    color: var(--color-accent, #ff4785);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .bubble-text {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-main, #f0f0f5);
    line-height: 1.4;
  }
</style>
