import { DEFAULT_LOCALE, type AppLocale } from "./locales";
import { en, type MessageCatalog } from "./messages/en";
import type { MessageShape } from "./messages/types";

export type RuntimeMessageCatalog = MessageShape<MessageCatalog>;
export type NonDefaultLocale = Exclude<AppLocale, typeof DEFAULT_LOCALE>;
export type LocaleCatalogImporter = () => Promise<RuntimeMessageCatalog>;

export interface LocaleCatalogLoader {
  load: (locale: AppLocale) => Promise<RuntimeMessageCatalog>;
  hasLoaded: (locale: AppLocale) => boolean;
}

/**
 * Creates a finite locale loader with English resident and one cached,
 * single-flight import for each non-default locale.
 */
export function createLocaleCatalogLoader(
  importers: Readonly<Record<NonDefaultLocale, LocaleCatalogImporter>>,
): LocaleCatalogLoader {
  const loaded = new Map<AppLocale, RuntimeMessageCatalog>([[DEFAULT_LOCALE, en]]);
  const inFlight = new Map<NonDefaultLocale, Promise<RuntimeMessageCatalog>>();

  function load(locale: AppLocale): Promise<RuntimeMessageCatalog> {
    const cached = loaded.get(locale);
    if (cached) return Promise.resolve(cached);

    const nonDefaultLocale = locale as NonDefaultLocale;
    const existing = inFlight.get(nonDefaultLocale);
    if (existing) return existing;

    let imported: Promise<RuntimeMessageCatalog>;
    try {
      imported = importers[nonDefaultLocale]();
    } catch (error) {
      imported = Promise.reject(error);
    }

    let request: Promise<RuntimeMessageCatalog>;
    request = imported
      .then((catalog) => {
        loaded.set(locale, catalog);
        return catalog;
      })
      .finally(() => {
        if (inFlight.get(nonDefaultLocale) === request) {
          inFlight.delete(nonDefaultLocale);
        }
      });
    inFlight.set(nonDefaultLocale, request);
    return request;
  }

  return {
    load,
    hasLoaded: (locale) => loaded.has(locale),
  };
}

const catalogLoader = createLocaleCatalogLoader({
  es: () => import("./messages/es").then((module) => module.es),
} satisfies Readonly<Record<NonDefaultLocale, LocaleCatalogImporter>>);

/** Loads the requested catalog without duplicating concurrent imports. */
export function loadLocaleCatalog(locale: AppLocale): Promise<RuntimeMessageCatalog> {
  return catalogLoader.load(locale);
}

/** Reports whether a locale catalog is resident in the current process. */
export function localeCatalogHasLoaded(locale: AppLocale): boolean {
  return catalogLoader.hasLoaded(locale);
}
