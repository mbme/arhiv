import {
  EditorState,
  Extension,
  Range,
  StateEffect,
  StateField,
  TransactionSpec,
} from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';
import { Decoration, DecorationSet, WidgetType } from '@codemirror/view';
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

function buildPreviews(state: EditorState, onNewRefs: (newRefs: DocumentId[]) => void) {
  const doc = state.doc;
  const refsCache = state.field(refsCacheField);

  const widgets: Range<Decoration>[] = [];
  const newRefs = new Set<DocumentId>();

  syntaxTree(state).iterate({
    enter: (cursor) => {
      const t = cursor.type.name;

      if (t === 'Link' || t === 'Image' || t === 'Autolink') {
        const urlNode = cursor.node.getChild('URL');
        if (!urlNode) {
          return;
        }

        const url = doc.sliceString(urlNode.from, urlNode.to);

        const id = tryParseRefUrl(url);
        if (!id) {
          return;
        }

        const refInfo = refsCache[id];

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

  // notify if there are new refs
  if (newRefs.size > 0) {
    onNewRefs([...newRefs]);
  }

  return widgets.length > 0 ? Decoration.set(widgets) : Decoration.none;
}

const createRefPreviewsField = (onNewRefs: (newRefs: DocumentId[]) => void) =>
  StateField.define<DecorationSet>({
    create(state) {
      return buildPreviews(state, onNewRefs);
    },

    update(previews, update) {
      const refsCacheUpdated =
        update.startState.field(refsCacheField) !== update.state.field(refsCacheField);

      const syntaxTreeChanged = syntaxTree(update.startState) !== syntaxTree(update.state);

      if (update.docChanged || refsCacheUpdated || syntaxTreeChanged) {
        return buildPreviews(update.state, onNewRefs);
      }

      return previews;
    },

    provide: (field) => EditorView.decorations.from(field),
  });

export function createRefLinkPlugin(
  initialRefsCache: RefsCache,
  onNewRefs: (newRefs: DocumentId[]) => void,
): Extension {
  return [
    refsCacheField.init(() => initialRefsCache), //
    createRefPreviewsField(onNewRefs),
  ];
}
