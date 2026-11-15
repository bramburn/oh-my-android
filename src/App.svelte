<script lang="ts">
  import { onMount } from 'svelte';
  import FaIcon from './lib/FaIcon.svelte';
  import DevicePanel from './lib/DevicePanel.svelte';
  import SetupScreen from './lib/SetupScreen.svelte';
  import { messageOf, setupReport, type SetupReport } from './lib/ipc';

  /** `loading` is the only state before the first report; after that the SDK decides the screen. */
  type View = 'loading' | 'error' | 'ready';

  let report = $state<SetupReport | null>(null);
  let view = $state<View>('loading');
  let errorMessage = $state('');
  /** Setup is forced when the SDK is missing, and reachable from the header once it is present. */
  let showSetup = $state(false);
  /** True only when the user opened setup by hand — then it stays open until they continue. */
  let userOpenedSetup = $state(false);

  const sdkPresent = $derived(report?.sdk !== null && report?.sdk !== undefined);

  async function load(): Promise<void> {
    try {
      applyReport(await setupReport());
    } catch (error) {
      report = null;
      view = 'error';
      errorMessage = messageOf(error);
    }
  }

  /**
   * The single rule for which screen is visible:
   * - no runnable adb → the guide, always (there is nothing else the app can do);
   * - adb found → never the guide, unless the user opened it from the header.
   */
  function applyReport(next: SetupReport): void {
    report = next;
    view = 'ready';
    if (next.sdk === null) {
      showSetup = true;
      userOpenedSetup = false;
    } else if (!userOpenedSetup) {
      // Found (or found again after Re-check / a folder choice): get out of the way.
      showSetup = false;
    }
  }

  onMount(() => {
    void load();
  });

  function continueToDevices(): void {
    if (report?.sdk === null) return; // guarded by the disabled button; never open an empty panel
    showSetup = false;
    userOpenedSetup = false;
  }

  function openSetup(): void {
    showSetup = true;
    userOpenedSetup = true;
    // Fresh truth every time the screen is opened: the user may have installed something meanwhile.
    void load();
  }
</script>

<header>
  <div class="title">Android Panel</div>
  {#if sdkPresent && !showSetup && view === 'ready'}
    <button
      type="button"
      class="header-btn"
      title="Setup and SDK folder"
      aria-label="Setup and SDK folder"
      onclick={openSetup}
    >
      <FaIcon name="screwdriver-wrench" />
    </button>
  {/if}
</header>

{#if view === 'loading'}
  <main>
    <div class="state">
      <h2>Checking your setup…</h2>
      <p>Looking for the Android SDK and adb.</p>
    </div>
  </main>
{:else if view === 'error'}
  <main>
    <div class="state">
      <h2>Could not read the setup</h2>
      <p>{errorMessage}</p>
      <button type="button" class="primary" onclick={load}>
        <FaIcon name="rotate" />
        Retry
      </button>
    </div>
  </main>
{:else if report !== null}
  {#if showSetup || report.sdk === null}
    <SetupScreen {report} onContinue={continueToDevices} onReport={applyReport} />
  {:else}
    <DevicePanel />
  {/if}
{/if}