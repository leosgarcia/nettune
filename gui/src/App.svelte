<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import './app.css';

  let tweaks = [];
  let loading = true;
  let dryRun = false;
  let toastMsg = null;

  async function loadTweaks() {
    loading = true;
    try {
      tweaks = await invoke('get_tweaks');
    } catch (e) {
      showToast(e, true);
    } finally {
      loading = false;
    }
  }

  async function applyTweak(id) {
    try {
      const msg = await invoke('apply_tweak', { id, dryRun });
      showToast(msg);
      if (!dryRun) await loadTweaks();
    } catch (e) {
      showToast(e, true);
    }
  }

  async function revertTweak(id) {
    try {
      const msg = await invoke('revert_tweak', { id, dryRun });
      showToast(msg);
      if (!dryRun) await loadTweaks();
    } catch (e) {
      showToast(e, true);
    }
  }

  async function revertAll() {
    try {
      const msg = await invoke('revert_all', { dryRun });
      showToast(msg);
      if (!dryRun) await loadTweaks();
    } catch (e) {
      showToast(e, true);
    }
  }

  function showToast(msg, isError = false) {
    toastMsg = { text: msg, isError };
    setTimeout(() => { toastMsg = null; }, 4000);
  }

  onMount(() => {
    loadTweaks();
  });
</script>

<main id="app">
  <header>
    <h1>NetTune</h1>
    <p class="subtitle">Otimização Nível SO para Redes de Baixa Latência</p>
  </header>

  <section class="card">
    <div class="header-actions">
      <h2>Ajustes Disponíveis</h2>
      <div class="controls">
        <label class="toggle-switch">
          <input type="checkbox" bind:checked={dryRun} class="switch-input" />
          <div class="switch-track">
            <div class="switch-thumb"></div>
          </div>
          Modo Dry-Run (Simulação)
        </label>
        <button class="danger" on:click={revertAll} title="Reverte todos os ajustes feitos pelo NetTune">
          Restaurar Tudo
        </button>
      </div>
    </div>

    {#if loading}
      <p style="text-align: center; color: var(--text-muted); padding: 2rem;">Carregando configurações nativas...</p>
    {:else}
      <div class="tweak-list">
        {#each tweaks as tweak (tweak.id)}
          <div class="tweak-item {tweak.active ? 'active' : ''}">
            <div class="tweak-info">
              <span class="tweak-name">{tweak.name}</span>
              <span class="tweak-desc">{tweak.description}</span>
            </div>
            <div class="actions">
              {#if tweak.active}
                <button class="danger" on:click={() => revertTweak(tweak.id)}>Reverter</button>
              {:else}
                <button class="success" on:click={() => applyTweak(tweak.id)}>Aplicar</button>
              {/if}
            </div>
          </div>
        {/each}
        {#if tweaks.length === 0}
          <p style="text-align: center; color: var(--text-muted);">Nenhum ajuste suportado neste Sistema Operacional.</p>
        {/if}
      </div>
    {/if}
  </section>

  {#if toastMsg}
    <div class="toast" style="border-color: {toastMsg.isError ? 'var(--danger)' : 'var(--success)'};">
      <span style="color: {toastMsg.isError ? 'var(--danger)' : 'var(--success)'}">
        {toastMsg.isError ? '⚠️' : '✅'}
      </span>
      <span>{toastMsg.text}</span>
    </div>
  {/if}
</main>
