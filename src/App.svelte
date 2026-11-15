<script lang="ts">
  import { onMount } from 'svelte';
  import {
    adbStart,
    adbStop,
    codeOf,
    listDevices,
    messageOf,
    type Device,
    type DeviceState,
  } from './lib/ipc';

  /** Explicit UI states — no screen may render stale data (`docs/windows-port/21-mvp-log.md` §2). */
  type Phase = 'loading' | 'sdkMissing' | 'error' | 'ready';

  let devices = $state<Device[]>([]);
  let phase = $state<Phase>('loading');
  let errorMessage = $state('');
  let busy = $state(false);

  const isEmpty = $derived(phase === 'ready' && devices.length === 0);

  /** One shared busy flag: concurrent taps are dropped, like upstream (`Sources/State/FeatureStore.swift:143`). */
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
      phase = codeOf(error) === 'SDK_NOT_FOUND' ? 'sdkMissing' : 'error';
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

  onMount(refresh);
</script>

<header>
  <div class="title">Android Panel</div>
</header>

<main>
  {#if phase === 'sdkMissing'}
    <div class="state">
      <h2>Android SDK not found</h2>
      <p>
        Install Android Studio or set <code>ANDROID_HOME</code>, then press Refresh.
      </p>
    </div>
  {:else if phase === 'error'}
    <div class="state">
      <h2>Could not read devices</h2>
      <p>{errorMessage}</p>
    </div>
  {:else if phase === 'loading'}
    <div class="state">
      <h2>Checking for devices…</h2>
      <p>Reading the adb device list.</p>
    </div>
  {:else if isEmpty}
    <div class="state">
      <h2>No devices connected.</h2>
      <p>Plug in a phone with USB debugging enabled, or start an emulator, then Refresh.</p>
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
            <span class="device-model">{device.model}</span>
            <span class={stateClass(device.state)}>{stateLabel(device.state)}</span>
          </div>
          <div class="device-serial">{device.serial}</div>
        </li>
      {/each}
    </ul>
  {/if}

  {#if phase === 'error' && errorMessage !== ''}
    <div class="banner" role="alert">
      <p>{errorMessage}</p>
      <button type="button" onclick={() => (errorMessage = '')}>Dismiss</button>
    </div>
  {/if}
</main>

<footer>
  <button type="button" class="primary" onclick={refresh} disabled={busy}>Refresh</button>
  <button type="button" onclick={() => runServer('start')} disabled={busy}>Start adb</button>
  <button type="button" onclick={() => runServer('stop')} disabled={busy}>Stop adb</button>
</footer>