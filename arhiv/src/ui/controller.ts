import { createContext, useContext } from 'react';
import { effect, signal } from '@preact/signals-core';
import { WorkspaceController } from 'Workspace/controller';

type Theme = 'light' | 'dark';

const prefersDarkThemeMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

const $prefersDarkTheme = signal<Theme>(prefersDarkThemeMediaQuery.matches ? 'dark' : 'light');

prefersDarkThemeMediaQuery.addEventListener('change', (e) => {
  $prefersDarkTheme.value = e.matches ? 'dark' : 'light';
});

export class AppController {
  readonly workspace = new WorkspaceController();

  readonly $theme = signal($prefersDarkTheme.value);

  constructor() {
    effect(() => {
      this.$theme.value = $prefersDarkTheme.value;
    });
  }

  toggleTheme() {
    const isDark = this.$theme.value === 'dark';

    this.$theme.value = isDark ? 'light' : 'dark';
  }
}

export const appController = new AppController();

const AppControllerContext = createContext(appController);

export function useAppController() {
  return useContext(AppControllerContext);
}
