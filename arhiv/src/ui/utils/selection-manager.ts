import { useEffect, useMemo, useRef } from 'react';
import { Inputs } from 'utils/hooks';

export class SelectionManager {
  constructor(private rootRef: React.RefObject<HTMLElement>) {}

  private getItems() {
    const root = this.rootRef.current;
    if (!root) {
      throw new Error('root element is missing');
    }

    return root.querySelectorAll<HTMLElement>('.sm-selectable');
  }

  activateItem = (index: number) => {
    this.getItems().forEach((el, i) => {
      const isActive = index === i;
      el.dataset['selected'] = isActive ? 'true' : '';

      if (isActive) {
        el.scrollIntoView({ block: 'nearest' });
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
  const rootRef = useRef<HTMLDivElement>(null);

  const selectionManager = useMemo(() => new SelectionManager(rootRef), [rootRef]);

  useEffect(() => {
    selectionManager.activateItem(0);
  }, [selectionManager, ...deps]); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    selectionManager,
    rootRef,
  };
}
