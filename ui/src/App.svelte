<script lang="ts">
  import Header from './components/Header.svelte';

  let statusMessage = $state('Shinoho Browser UI bereit');
</script>

<div class="shinoho-app" data-testid="shinoho-app">
  <Header title="Shinoho" />
  <main class="content">
    <div class="welcome-card">
      <h1>Shinoho</h1>
      <p class="status" data-testid="status-message">{statusMessage}</p>
    </div>
  </main>
</div>

<style>
  .shinoho-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .welcome-card {
    text-align: center;
    padding: 32px 48px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
  }

  .welcome-card h1 {
    font-size: 28px;
    margin-bottom: 8px;
    color: var(--color-accent);
  }

  .status {
    color: var(--color-text-muted);
    font-size: 14px;
  }
</style>
