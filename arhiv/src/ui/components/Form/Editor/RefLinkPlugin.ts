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
import { isErasedDocument } from 'utils/schema';
import { DocumentId, DocumentType } from 'dto';
import { RefsCache } from 'controller';
import { getImageUrl, getPreviewType } from 'components/AssetPreview';

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
    dom.className = 'font-semibold var-active-color hover:var-active-color-hover cursor-default';
    dom.classList.toggle('line-through', isErasedDocument(this.documentType));
    dom.classList.toggle('text-slate-700/50', isErasedDocument(this.documentType));

    this.updateEl(dom);

    return dom;
  }
}

// widget that displays referenced assets' image
class RefImageWidget extends WidgetType {
  constructor(
    readonly id: DocumentId,
    readonly size: number,
  ) {
    super();
  }

  override eq(other: RefImageWidget): boolean {
    return this.id === other.id && this.size === other.size;
  }

  override updateDOM(dom: HTMLImageElement, view: EditorView): boolean {
    this.updateEl(dom, view);
    return true;
  }

  private updateEl(el: HTMLImageElement, view: EditorView) {
    el.src = getImageUrl(this.id, this.size);
    el.onload = () => {
      view.requestMeasure();
    };
  }

  toDOM(view: EditorView) {
    const dom = document.createElement('img');
    dom.className = 'block h-64 w-auto max-w-full mx-auto object-contain';

    this.updateEl(dom, view);

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
          const refDecoration = Decoration.replace({
            widget: new RefLinkWidget(id, refInfo.documentType, refInfo.title),
          });
          widgets.push(refDecoration.range(urlNode.from, urlNode.to));

          const isImageTag = t === 'Image';
          const isImageAsset = getPreviewType(refInfo.documentType, refInfo.data);

          if (isImageTag && isImageAsset) {
            const size = refInfo.data['size'] as number;
            const refImageDecoration = Decoration.widget({
              widget: new RefImageWidget(id, size),
              side: -1,
              block: true,
            }).range(cursor.node.to); // insert *after* the markdown link
            widgets.push(refImageDecoration);
          }
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
