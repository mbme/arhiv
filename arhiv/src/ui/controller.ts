import { createContext, useContext } from 'react';
import { signal } from '@preact/signals-core';
import { DocumentData, DocumentId, DocumentType } from './dto';
import { throttle } from './utils/index';
import { RPC } from './utils/network';
import { useSignal } from './utils/hooks';
import { WorkspaceController } from './Workspace/controller';
import { DocumentChangeEvent } from './Workspace/documentChangeUtils';

export type Theme = 'light' | 'dark';

const prefersDarkThemeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
const getPreferredTheme = (): Theme => (prefersDarkThemeMediaQuery.matches ? 'dark' : 'light');

export type RefInfo = {
  documentType: DocumentType;
  title: string;
  data: DocumentData;
};

export type RefsCache = Record<DocumentId, RefInfo>;

export class AppController {
  readonly workspace = new WorkspaceController();

  readonly $theme = signal<Theme>(getPreferredTheme());

  readonly $refsCache = signal<RefsCache>({});
  private refsToFetch = new Set<DocumentId>();

  constructor() {
    prefersDarkThemeMediaQuery.addEventListener('change', (e) => {
      this.$theme.value = e.matches ? 'dark' : 'light';
    });

    document.addEventListener(DocumentChangeEvent.EVENT_NAME, (e) => {
      const ids = (e as DocumentChangeEvent).detail;

      // re-fetch updated documents
      const cachedIds = [...ids].filter((id) => this.$refsCache.peek()[id]);
      void this.fetchRefs(cachedIds);
    });
  }

  toggleTheme() {
    const isDark = this.$theme.value === 'dark';

    this.$theme.value = isDark ? 'light' : 'dark';
  }

  private throttledFetchRefs = throttle(async () => {
    if (this.refsToFetch.size === 0) {
      return;
    }

    const ids = [...this.refsToFetch];

    const result = await RPC.GetDocuments({ ids, ignoreMissing: true });

    const newRefs: RefsCache = {
      ...this.$refsCache.value,
    };
    for (const document of result.documents) {
      newRefs[document.id] = {
        documentType: document.documentType,
        title: document.title,
        data: document.data,
      };
    }

    this.refsToFetch.clear();
    this.$refsCache.value = newRefs;
  }, 300);

  async fetchRefs(ids: DocumentId[]) {
    for (const id of ids) {
      this.refsToFetch.add(id);
    }

    if (this.refsToFetch.size > 0) {
      await this.throttledFetchRefs();
    }
  }
}

export const appController = new AppController();

const AppControllerContext = createContext(appController);

export function useAppController() {
  return useContext(AppControllerContext);
}

export function useCachedRef(id: DocumentId): RefInfo {
  const app = useAppController();

  const refsCache = useSignal(app.$refsCache);
  const info = refsCache[id];

  if (!info) {
    // oxlint-disable-next-line typescript/only-throw-error
    throw app.fetchRefs([id]);
  }

  return info;
}
