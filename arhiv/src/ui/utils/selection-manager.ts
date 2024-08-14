import { useEffect, useMemo, useState } from 'react';
import { effect, signal } from '@preact/signals-core';
import { Inputs, useKeydown } from 'utils/hooks';

const SELECTABLE_ELEMENT_SELECTOR = '.sm-selectable';

export class SelectionManager {
  private $rootEl = signal<HTMLElement | null>(null);

  constructor() {
    // activate selectable element on hover
    effect(() => {
      const rootEl = this.$rootEl.value;
      if (!rootEl) {
        return;
      }

      const onMouseOver = (e: MouseEvent) => {
        if (e.target instanceof HTMLElement && e.target.matches(SELECTABLE_ELEMENT_SELECTOR)) {
          this.activateElement(e.target);
        }
      };

      rootEl.addEventListener('mouseover', onMouseOver, { passive: true });

      return () => {
        rootEl.removeEventListener('mouseover', onMouseOver);
      };
    });
  }

  setRootEl(rootEl: HTMLElement | null): void {
    this.$rootEl.value = rootEl;
  }

  hasRootEl(): boolean {
    return !!this.$rootEl.value;
  }

  private getItems() {
    const root = this.$rootEl.value;
    if (!root) {
      throw new Error('root element is missing');
    }

    return root.querySelectorAll<HTMLElement>(SELECTABLE_ELEMENT_SELECTOR);
  }

  activateItem = (index: number) => {
    this.getItems().forEach((el, i) => {
      const isActive = index === i;
      el.dataset['selected'] = isActive ? 'true' : '';

      if (isActive) {
        el.scrollIntoView({ block: 'nearest', behavior: 'instant' });
      }
    });
  };

  activateNextItem = (increment = true) => {
    const items = Array.from(this.getItems());

    if (items.length === 0) {
      return;
    }

    const index = items.findIndex((el) => el.dataset['selected'] === 'true');

    let newIndex = index + (increment ? 1 : -1);
    if (newIndex >= items.length) {
      newIndex = 0;
    }
    if (newIndex < 0) {
      newIndex = items.length - 1;
    }

    this.activateItem(newIndex);
  };

  activateElement = (el: HTMLElement) => {
    const items = Array.from(this.getItems());
    const index = items.indexOf(el);

    if (index === -1) {
      throw new Error("Can't activate element: not selectable");
    }

    this.activateItem(index);
  };

  clickActive = () => {
    const items = Array.from(this.getItems());

    const activeItem = items.find((el) => el.dataset['selected'] === 'true');

    activeItem?.click();
  };

  handleKey(key: string): boolean {
    switch (key) {
      case 'Enter': {
        this.clickActive();

        return true;
      }
      case 'ArrowUp': {
        this.activateNextItem(false);

        return true;
      }
      case 'ArrowDown': {
        this.activateNextItem(true);

        return true;
      }
      default: {
        return false;
      }
    }
  }
}

export function useSelectionManager(deps: Inputs = []) {
  const [rootEl, setRootEl] = useState<HTMLElement | null>(null);

  const selectionManager = useMemo(() => new SelectionManager(), []);

  useEffect(() => {
    selectionManager.setRootEl(rootEl);
  }, [selectionManager, rootEl]);

  useEffect(() => {
    if (selectionManager.hasRootEl()) {
      selectionManager.activateItem(0);
    }
  }, [selectionManager, ...deps]); // eslint-disable-line react-hooks/exhaustive-deps

  useKeydown(rootEl, (e) => {
    const handled = selectionManager.handleKey(e.code);

    if (handled) {
      e.preventDefault();
    }
  });

  return {
    selectionManager,
    setRootEl,
  };
}
