<script lang="ts">
  import FaIcon from './FaIcon.svelte';
  import {
    DOCS,
    chooseSdkFolder,
    messageOf,
    openDocs,
    sdkSourceLabel,
    type SetupReport,
  } from './ipc';

  interface Props {
    report: SetupReport;
    onContinue: () => void;
    onReport: (report: SetupReport) => void;
  }

  let { report, onContinue, onReport }: Props = $props();

  let busy = $state(false);
  let error = $state('');

  const sdkOk = $derived(report.sdk !== null);
  const adbOk = $derived(report.adbVersion !== null);
  const emulatorOk = $derived(report.emulator !== null);

  async function pickFolder(): Promise<void> {
    if (busy) return;
    busy = true;
    error = '';
    try {
      // Cancelling resolves to the unchanged report, so no cancel UI is needed here.
      onReport(await chooseSdkFolder());
    } catch (problem) {
      error = messageOf(problem);
    } finally {
      busy = false;
    }
  }

  async function open(url: (typeof DOCS)[keyof typeof DOCS]): Promise<void> {
    try {
      await openDocs(url);
    } catch (problem) {
      error = messageOf(problem);
    }
  }
</script>

<main class="setup">
  <h1 class="setup-title">Set up your Android tooling</h1>
  <p class="setup-intro">
    This app drives <code>adb</code>, which comes with the Android SDK. It checks what is installed and
    tells you what to do — it never downloads or installs anything for you.
  </p>

  {#if report.warning}
    <div class="banner" role="alert">
      <FaIcon name="triangle-exclamation" />
      <p>{report.warning}</p>
    </div>
  {/if}

  <ul class="checks">
    <li class="check" class:ok={report.studio !== null}>
      <div class="check-icon">
        <FaIcon name={report.studio !== null ? 'circle-check' : 'circle-xmark'} />
      </div>
      <div class="check-body">
        <h2 class="check-title">Android Studio</h2>
        {#if report.studio}
          <p class="check-path">{report.studio}</p>
        {:else}
          <p class="check-hint">
            Easiest way to get the SDK and the emulator. Install it, then press Re-check.
          </p>
          <div class="check-actions">
            <button type="button" onclick={() => open(DOCS.studio)}>
              <FaIcon name="arrow-up-right-from-square" />
              Open download page
            </button>
          </div>
        {/if}
      </div>
    </li>

    <li class="check" class:ok={sdkOk}>
      <div class="check-icon">
        <FaIcon name={sdkOk ? 'circle-check' : 'circle-xmark'} />
      </div>
      <div class="check-body">
        <h2 class="check-title">Android SDK (platform-tools)</h2>
        {#if report.sdk}
          <p class="check-path">{report.sdk.root}</p>
          <p class="check-meta">found via {sdkSourceLabel(report.sdk.source)}</p>
        {:else}
          <p class="check-hint">
            Installed by Android Studio's setup wizard, or on its own as “platform-tools”. If you have it
            somewhere unusual, point the app at the folder that contains
            <code>platform-tools</code>.
          </p>
          <p class="check-meta">
            {#if report.envHome}
              ANDROID_HOME is set to {report.envHome}, but no adb was found there.
            {:else}
              No ANDROID_HOME set, and the default locations are empty.
            {/if}
          </p>
          <div class="check-actions">
            <button type="button" onclick={() => open(DOCS.adb)}>
              <FaIcon name="arrow-up-right-from-square" />
              Open platform-tools docs
            </button>
            <button type="button" class="primary" onclick={pickFolder} disabled={busy}>
              <FaIcon name="folder-open" />
              Choose SDK folder…
            </button>
          </div>
        {/if}
      </div>
    </li>

    <li class="check" class:ok={adbOk}>
      <div class="check-icon">
        <FaIcon name={adbOk ? 'circle-check' : sdkOk ? 'circle-xmark' : 'circle-info'} />
      </div>
      <div class="check-body">
        <h2 class="check-title">adb</h2>
        {#if report.adbVersion}
          <p class="check-path">{report.adbVersion}</p>
        {:else if sdkOk}
          <p class="check-hint">
            <code>adb.exe</code> is there but did not answer <code>version</code>. It may be blocked by
            security software or corrupted — reinstall platform-tools.
          </p>
        {:else}
          <p class="check-hint">Covered by the Android SDK row above.</p>
        {/if}
      </div>
    </li>

    <li class="check optional" class:ok={emulatorOk}>
      <div class="check-icon">
        <FaIcon name={emulatorOk ? 'circle-check' : 'circle-info'} />
      </div>
      <div class="check-body">
        <h2 class="check-title">Emulator</h2>
        {#if report.emulator}
          <p class="check-path">{report.emulator}</p>
        {:else}
          <p class="check-hint">
            Optional for a physical phone. Install it through Studio → SDK Manager → Android Emulator.
          </p>
        {/if}
      </div>
    </li>

    <li class="check optional" class:ok={report.avdCount > 0}>
      <div class="check-icon">
        <FaIcon name={report.avdCount > 0 ? 'circle-check' : 'circle-info'} />
      </div>
      <div class="check-body">
        <h2 class="check-title">Virtual devices</h2>
        {#if report.avdCount > 0}
          <p class="check-path">
            {report.avdCount}
            {report.avdCount === 1 ? 'virtual device' : 'virtual devices'} configured
          </p>
        {:else}
          <p class="check-hint">
            None configured. Create one in Studio → Device Manager. Not needed for a physical phone.
          </p>
          <div class="check-actions">
            <button type="button" onclick={() => open(DOCS.avds)}>
              <FaIcon name="arrow-up-right-from-square" />
              Open AVD docs
            </button>
          </div>
        {/if}
      </div>
    </li>
  </ul>

  <div class="hint-block">
    <p>
      <strong>Using a physical phone?</strong>
      Enable Developer options → USB debugging, then accept the prompt on the phone.
    </p>
    <button type="button" onclick={() => open(DOCS.usb)}>
      <FaIcon name="arrow-up-right-from-square" />
      Open USB debugging guide
    </button>
  </div>

  {#if error}
    <div class="banner" role="alert">
      <FaIcon name="triangle-exclamation" />
      <p>{error}</p>
      <button type="button" onclick={() => (error = '')}>Dismiss</button>
    </div>
  {/if}

  <p class="setup-note">
    This app never installs anything for you — it only checks and guides. Icons: Font Awesome Free
    (CC BY 4.0), bundled locally.
  </p>
</main>

<footer>
  <button type="button" onclick={() => onReport(report)} disabled={busy}>
    <FaIcon name="rotate" />
    Re-check
  </button>
  <button type="button" class="primary" onclick={onContinue} disabled={busy || !sdkOk}>
    <FaIcon name="mobile-screen" />
    Continue to devices
  </button>
</footer>