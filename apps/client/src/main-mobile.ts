import { Temporal } from "@js-temporal/polyfill";
import "@fontsource-variable/inter";
import { mount } from "svelte";
import "./app.css";
import { ensureConfigLoaded, flushConfig } from "$lib/vault/config";
import { getActiveVaultInfo } from "$lib/vault/state";
import {
  getLocalization,
  initializeLocalizationFromConfig,
} from "$lib/i18n/translator.svelte";
import { DEFAULT_LANGUAGE_PREFERENCE } from "$lib/i18n/locales";
import {
  clearPreVaultLanguagePreference,
  readPreVaultLanguagePreference,
} from "$lib/i18n/pre-vault-language";
import { applyPlatformProfileToDocument } from "$lib/platform";
import { installModalKeyboardRouter } from "$lib/modal-focus";
import { hydrateUserThemes } from "$lib/stores/theme.svelte";

(globalThis as unknown as { Temporal: typeof Temporal }).Temporal = Temporal;
applyPlatformProfileToDocument();
installModalKeyboardRouter();

function safeStorage(): Storage | undefined {
  try {
    return window.localStorage;
  } catch {
    return undefined;
  }
}

async function applyPreVaultLanguagePreference(): Promise<void> {
  const storage = safeStorage();
  const preference = readPreVaultLanguagePreference(storage);
  if (!preference) return;
  const applied = await getLocalization().setLanguagePreference(preference);
  if (!applied) return;
  await flushConfig();
  clearPreVaultLanguagePreference(storage);
}

async function mountMobileVaultError(message: string) {
  const { default: MobileVaultErrorView } = await import(
    "$lib/components/mobile/MobileVaultErrorView.svelte"
  );
  return mount(MobileVaultErrorView, {
    target: document.getElementById("app")!,
    props: {
      message,
      onRetry: () => window.location.reload(),
    },
  });
}

async function mountMobileVaultSetup(initialError: string | null) {
  const { default: MobileVaultSetupView } = await import(
    "$lib/components/mobile/MobileVaultSetupView.svelte"
  );
  return mount(MobileVaultSetupView, {
    target: document.getElementById("app")!,
    props: {
      initialError,
      onReady: () => window.location.reload(),
    },
  });
}

async function mountMobileFocusOnboarding() {
  const { default: MobileFocusOnboarding } = await import(
    "$lib/components/mobile/MobileFocusOnboarding.svelte"
  );
  return mount(MobileFocusOnboarding, {
    target: document.getElementById("app")!,
    props: {
      onComplete: () => window.location.reload(),
    },
  });
}

const appPromise = (async () => {
  const preVaultPreference = readPreVaultLanguagePreference(safeStorage());
  await getLocalization().setLanguagePreference(
    preVaultPreference ?? DEFAULT_LANGUAGE_PREFERENCE,
    { persist: false },
  );

  try {
    const activeVault = await getActiveVaultInfo();
    if (!activeVault) return mountMobileVaultSetup(null);
    await ensureConfigLoaded();
    await initializeLocalizationFromConfig();
    await applyPreVaultLanguagePreference();
    await hydrateUserThemes();
    const { mobileFocusOnboardingCompleted } = await import(
      "$lib/scheduling/mobile-background-execution"
    );
    if (!mobileFocusOnboardingCompleted(safeStorage())) {
      return mountMobileFocusOnboarding();
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return mountMobileVaultError(message);
  }

  const { default: MobileApp } = await import("./MobileApp.svelte");
  return mount(MobileApp, {
    target: document.getElementById("app")!,
  });
})();

export default appPromise;
