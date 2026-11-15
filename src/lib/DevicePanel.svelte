<script lang="ts">
  import FaIcon from './FaIcon.svelte';
  import { onMount } from 'svelte';
  import {
    adbStart,
    adbStop,
    listDevices,
    messageOf,
    type Device,
    type DeviceState,
  } from './ipc';

  type Phase = 'loading' | 'error' | 'ready';

  let devices = $state<Device[]>([]);
  let phase = $state<Phase>('loading');
  let errorMessage = $state('');
  let busy = $state(false);

  const isEmpty = $derived(phase === 'ready' && devices.length === 0);

  function stateLabel(state: DeviceState): string {
    switch (state) {
      case 'device':
        return 'ready';
      case 'unauthorized':
        return 'unauthorized';
      case 'offline':
        return 'offline';
      default:
        return 'other';
    }
  }

  function stateClass(state: DeviceState): string {
    if (state === 'device') return 'badge ready';
    if (state === 'other') return 'badge';
    return 'badge attention';
  }

  function isEmulator(serial: string): boolean {
    return serial.startsWith('emulator-');
  }

  async function refresh(): Promise<void> {
    if (busy) return;
    busy = true;
    errorMessage = '';
    if (devices.length === 0) phase = 'loading';
    try {
      devices = await listDevices();
      phase = 'ready';
    } catch (error) {
      // Override O1: a failed refresh must never leave rows on screen that look live.
      devices = [];
      phase = 'error';
      errorMessage = messageOf(error);
    } finally {
      busy = false;
    }
  }

  async function runServer(action: 'start' | 'stop'): Promise<void> {
    if (busy) return;
    if (
      action === 'stop' &&
      !confirm('Stop the adb server? Connected devices will disconnect.')
    ) {
      return;
    }
    busy = true;
    errorMessage = '';
    try {
      if (action === 'start') {
        await adbStart();
      } else {
        await adbStop();
      }
    } catch (error) {
      devices = [];
      phase = 'error';
      errorMessage = messageOf(error);
      busy = false;
      return;
    }
    busy = false;
    await refresh();
  }

  // Load once on mount. `$effect` would re-run on every state change `refresh` reads.
  onMount(() => {
    void refresh();
  });
</script>

<main>
  {#if phase === 'error'}
    <div class="state">
      <h2>Could not read devices</h2>
      <p>{errorMessage}</p>
      <button type="button" class="primary" onclick={refresh} disabled={busy}>
        <FaIcon name="rotate" />
        Retry
      </button>
    </div>
  {:else if phase === 'loading'}
    <div class="state">
      <h2>Checking for devices…</h2>
      <p>Reading the adb device list.</p>
    </div>
  {:else if isEmpty}
    <div class="state">
      <h2>No devices connected.</h2>
      <p>Plug in a phone with USB debugging enabled, or start an emulator, then press Refresh.</p>
    </div>
  {:else}
    <p class="count">
      {devices.length}
      {devices.length === 1 ? 'device' : 'devices'}
    </p>
    <ul class="devices">
      {#each devices as device (device.serial)}
        <li class="device">
          <div class="device-top">
            <span class="device-name">
              <FaIcon name={isEmulator(device.serial) ? 'display' : 'mobile-screen'} />
              <span class="device-model">{device.model}</span>
            </span>
            <span class={stateClass(device.state)}>{stateLabel(device.state)}</span>
          </div>
          <div class="device-serial">{device.serial}</div>
          {#if device.state === 'unauthorized'}
            <p class="device-hint">Accept the USB debugging prompt on the phone.</p>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if phase === 'error' && errorMessage !== ''}
    <div class="banner" role="alert">
      <FaIcon name="triangle-exclamation" />
      <p>{errorMessage}</p>
      <button type="button" onclick={() => (errorMessage = '')}>Dismiss</button>
    </div>
  {/if}
</main>

<footer>
  <button type="button" class="primary" onclick={refresh} disabled={busy}>
    <FaIcon name="rotate" />
    Refresh
  </button>
  <button type="button" onclick={() => runServer('start')} disabled={busy}>
    <FaIcon name="play" />
    Start adb
  </button>
  <button type="button" onclick={() => runServer('stop')} disabled={busy}>
    <FaIcon name="stop" />
    Stop adb
  </button>
</footer>
