import { Extension, Range, StateEffect, StateField, TransactionSpec } from '@codemirror/state';
import { EditorView, ViewUpdate } from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';
import { Decoration, DecorationSet, ViewPlugin, WidgetType } from '@codemirror/view';
import { tryParseRefUrl } from 'utils/markup';
import { DocumentId, DocumentType } from 'dto';
import { RefsCache } from 'controller';

// effect to update refs cache state field
const setRefsCache = StateEffect.define<RefsCache>();

export function updateRefsCache(refsCache: RefsCache): TransactionSpec {
  return {
    effects: [setRefsCache.of(refsCache)],
  };
}

// state field that contains refs cache
const refsCacheField = StateField.define<RefsCache>({
  create() {
    return {};
  },

  update(refsCache, transaction) {
    let newRefsCache = refsCache;

    for (const e of transaction.effects) {
      if (e.is(setRefsCache)) {
        newRefsCache = e.value;
      }
    }

    return newRefsCache;
  },
});

// widget that displays document title instead of ref link
class RefLinkWidget extends WidgetType {
  constructor(
    readonly id: DocumentId,
    readonly documentType: DocumentType,
    readonly title: string,
  ) {
    super();
  }

  override eq(other: RefLinkWidget): boolean {
    return (
      this.id === other.id && this.documentType === other.documentType && this.title === other.title
    );
  }

  override updateDOM(dom: HTMLElement): boolean {
    this.updateEl(dom);
    return true;
  }

  private updateEl(el: HTMLElement) {
    el.textContent = `${this.documentType}:${this.title}`;
    el.title = this.id;
  }

  toDOM() {
    const dom = document.createElement('span');
    dom.className = 'var-active-color hover:var-active-color-hover cursor-default';

    this.updateEl(dom);

    return dom;
  }
}

export function createRefLinkPlugin(
  initialRefsCache: RefsCache,
  onNewRefs: (newRefs: DocumentId[]) => void,
): Extension {
  const refLinkPlugin = ViewPlugin.fromClass(
    class RefLinkPlugin {
      refsCache: RefsCache;
      decorations: DecorationSet = Decoration.set([]);

      constructor(view: EditorView) {
        this.refsCache = view.state.field(refsCacheField);
        this.buildDecorations(view);
      }

      update(update: ViewUpdate) {
        const updateRefsCache = update.state.field(refsCacheField);

        const refsCacheUpdated = this.refsCache !== updateRefsCache;

        if (refsCacheUpdated) {
          this.refsCache = updateRefsCache;
        }

        if (
          update.docChanged ||
          update.viewportChanged ||
          refsCacheUpdated ||
          syntaxTree(update.startState) != syntaxTree(update.state)
        ) {
          this.buildDecorations(update.view);
        }
      }

      buildDecorations(view: EditorView) {
        const doc = view.state.doc;

        const widgets: Range<Decoration>[] = [];
        const newRefs = new Set<DocumentId>();

        for (const { from, to } of view.visibleRanges) {
          syntaxTree(view.state).iterate({
            from,
            to,
            enter: (cursor) => {
              const t = cursor.type.name;

              if (t == 'Link' || t == 'Image' || t == 'Autolink') {
                const urlNode = cursor.node.getChild('URL');
                if (!urlNode) {
                  return;
                }

                const url = doc.sliceString(urlNode.from, urlNode.to);

                const id = tryParseRefUrl(url);
                if (!id) {
                  return;
                }

                const refInfo = this.refsCache[id];

                if (refInfo) {
                  const decoration = Decoration.replace({
                    widget: new RefLinkWidget(id, refInfo.documentType, refInfo.title),
                  });

                  widgets.push(decoration.range(urlNode.from, urlNode.to));
                } else {
                  newRefs.add(id);
                }
              }
            },
          });
        }

        // notify if there are new refs
        if (newRefs.size > 0) {
          onNewRefs([...newRefs]);
        }

        this.decorations = Decoration.set(widgets);
      }
    },
    {
      decorations: (v) => v.decorations,
    },
  );

  return [
    refsCacheField.init(() => initialRefsCache), //
    refLinkPlugin,
  ];
}
