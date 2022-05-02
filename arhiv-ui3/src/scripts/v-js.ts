import { sum } from './utils';

export type DirectiveName = `v-${string}`;
export type Directive = (el: Element, arg: string) => void;
type Directives = Record<DirectiveName, Directive>;

class DirectivesProcessor {
  private _registries: Record<DirectiveName, WeakSet<Element>>;

  constructor(private _directives: Directives) {
    this._registries = {};
    for (const directiveName of Object.keys(_directives)) {
      this._registries[directiveName as DirectiveName] = new WeakSet();
    }
  }

  processElements(): Record<DirectiveName, number> {
    const result: Record<DirectiveName, number> = {};

    for (const directiveName of Object.keys(this._directives)) {
      const processed = this._processDirective(directiveName as DirectiveName);
      result[directiveName as DirectiveName] = processed;
    }

    return result;
  }

  private _processDirective(directiveName: DirectiveName): number {
    const directive = this._directives[directiveName];
    const registry = this._registries[directiveName];

    let directivesProcessed = 0;

    for (const el of document.querySelectorAll(`[${directiveName}]`)) {
      if (registry.has(el)) {
        continue;
      }

      const attr = el.attributes.getNamedItem(directiveName);
      if (!attr) {
        throw new Error(`element doesn't have attribute "${directiveName}"`);
      }

      const value = attr.value;

      try {
        directive(el, value);

        directivesProcessed += 1;

        registry.add(el);
      } catch (e) {
        console.error(
          'Failed to execute directive "%s": %s\n script: %s\n',
          directiveName,
          e,
          value,
          el
        );
      }
    }

    return directivesProcessed;
  }
}

const DIRECTIVES: Directives = {};

export function registerVDirective(directiveName: DirectiveName, directive: Directive) {
  DIRECTIVES[directiveName] = directive;
}

export function init_V_JS(observeChanges = false): void {
  console.debug('[v]: registered %s directives', Object.keys(DIRECTIVES).length);

  const processor = new DirectivesProcessor(DIRECTIVES);

  {
    const result = processor.processElements();
    const total = Object.values(result).reduce(sum, 0);

    console.debug('[v]: processed %s directives on init %o', total, result);
  }

  if (!observeChanges) {
    return;
  }

  const observer = new MutationObserver((mutations) => {
    const hasNewNodes = mutations.find((mutation) => mutation.addedNodes.length > 0);

    if (!hasNewNodes) {
      return;
    }

    const result = processor.processElements();
    const total = Object.values(result).reduce(sum, 0);

    console.debug('[v]: processed %s directives on dom mutation %o', total, result);
  });

  observer.observe(document.body, {
    subtree: true,
    childList: true,
  });
}
