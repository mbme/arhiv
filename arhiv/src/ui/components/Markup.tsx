import { createElement, forwardRef, useImperativeHandle, useRef } from 'react';
import { cx, Obj } from 'utils';
import { useSuspenseQuery } from 'utils/suspense';
import { JSXElement, JSXRef } from 'utils/jsx';
import { tryParseRefUrl } from 'utils/markup';
import { MarkupElement, throwBadMarkupElement, Range, DocumentId } from 'dto';
import { useCachedRef } from 'controller';
import { Link } from 'components/Link';
import { Ref } from 'components/Ref';
import { AssetPreviewBlock, canPreview } from 'components/AssetPreview';

function extractText(children: MarkupElement[]): string {
  return children
    .flatMap((el) => {
      if (el.typeName === 'Text') {
        return el.value;
      }

      if ('children' in el) {
        return extractText(el.children);
      }

      return null;
    })
    .filter((item) => Boolean(item))
    .join(' ');
}

function rangeToString(range: Range): string {
  return `${range.start}-${range.end}`;
}

function markupElementToJSX(el: MarkupElement, ref?: JSXRef<HTMLDivElement>): JSXElement {
  switch (el.typeName) {
    case 'Document': {
      return (
        <div className="markup w-full" ref={ref}>
          {el.children.map((child) => markupElementToJSX(child))}
        </div>
      );
    }
    case 'Text': {
      return (
        <span
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.value}
        </span>
      );
    }
    case 'Code': {
      return (
        <code
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.value}
        </code>
      );
    }
    case 'Html': {
      return (
        <span
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
          dangerouslySetInnerHTML={{ __html: el.value }}
        />
      );
    }
    case 'SoftBreak': {
      return (
        <br
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        />
      );
    }
    case 'HardBreak': {
      return (
        <br
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        />
      );
    }
    case 'Rule': {
      return (
        <hr
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        />
      );
    }
    case 'Paragraph': {
      return (
        <p
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </p>
      );
    }
    case 'Heading': {
      return createElement(
        el.level.toLowerCase(),
        {
          key: rangeToString(el.range),
          'data-range-start': el.range.start,
          'data-range-end': el.range.end,
        } as Obj<unknown>,
        ...el.children.map((child) => markupElementToJSX(child)),
      );
    }
    case 'BlockQuote': {
      return (
        <blockquote
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </blockquote>
      );
    }
    case 'CodeBlock': {
      // TODO handle kind
      return (
        <pre key={rangeToString(el.range)}>
          <code data-range-start={el.range.start} data-range-end={el.range.end}>
            {el.children.map((child) => markupElementToJSX(child))}
          </code>
        </pre>
      );
    }
    case 'List': {
      return createElement(
        el.first_item_number == null ? 'ul' : 'ol',
        {
          key: rangeToString(el.range),
          'start': el.first_item_number ?? undefined,
          'data-range-start': el.range.start,
          'data-range-end': el.range.end,
        },

        ...el.children.map((child) => markupElementToJSX(child)),
      );
    }
    case 'TaskListMarker': {
      return (
        <input
          key={rangeToString(el.range)}
          type="checkbox"
          className="mr-2"
          defaultChecked={el.checked}
          onClick={(e) => {
            e.stopPropagation();
            e.preventDefault();
          }}
          tabIndex={-1}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        />
      );
    }
    case 'ListItem': {
      return (
        <li
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </li>
      );
    }
    case 'FootnoteReference': {
      throw new Error('NYI');
    }
    case 'FootnoteDefinition': {
      throw new Error('NYI');
    }
    case 'Table': {
      // TODO handle alignments

      const head = el.children[0]!;
      if (head.typeName !== 'TableHead') {
        throw new Error(`Expected TableHead, got ${head.typeName}`);
      }

      return (
        <table
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {markupElementToJSX(head)}

          <tbody>{el.children.slice(1).map((child) => markupElementToJSX(child))}</tbody>
        </table>
      );
    }
    case 'TableHead': {
      return (
        <thead key={rangeToString(el.range)}>
          <tr data-range-start={el.range.start} data-range-end={el.range.end}>
            {el.children.map((col, index) => {
              if (col.typeName !== 'TableCell') {
                throw new Error(`Expected TableCell, got ${col.typeName}`);
              }

              return <th key={index}>{col.children.map((child) => markupElementToJSX(child))}</th>;
            })}
          </tr>
        </thead>
      );
    }
    case 'TableRow': {
      return (
        <tr
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </tr>
      );
    }
    case 'TableCell': {
      return (
        <td
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </td>
      );
    }
    case 'Emphasis': {
      return (
        <em
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </em>
      );
    }
    case 'Strong': {
      return (
        <strong
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </strong>
      );
    }
    case 'Strikethrough': {
      return (
        <s
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
        </s>
      );
    }
    case 'Link':
    case 'Image': {
      let description = extractText(el.children);

      const id = tryParseRefUrl(el.url);
      if (id) {
        const preview = el.typeName === 'Image';

        if (el.link_type === 'Autolink') {
          description = '';
        }

        return (
          <span
            key={rangeToString(el.range)}
            data-range-start={el.range.start}
            data-range-end={el.range.end}
            className={cx({
              'inline-block w-full': el.typeName === 'Image',
            })}
          >
            <RefContainer id={id} assetPreview={preview} description={description} />
          </span>
        );
      }

      // TODO handle link_type?
      return (
        <span
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          <Link url={el.url}>{description}</Link>
        </span>
      );
    }
  }

  throwBadMarkupElement(el);
}

function getFirstVisiblePosInMarkup(
  viewportEl: HTMLElement,
  markupEl: HTMLElement,
): number | undefined {
  const MIN_EL_HEIGHT_PX = 12;

  const viewportBounding = viewportEl.getBoundingClientRect();
  const viewportTop = viewportBounding.top;
  const viewportBottom = viewportTop + viewportBounding.height;

  for (const el of markupEl.querySelectorAll<HTMLElement>('[data-range-start]')) {
    const elBounding = el.getBoundingClientRect();
    const elTop = elBounding.top;
    const elBottom = elTop + elBounding.height;

    if (elBottom - MIN_EL_HEIGHT_PX > viewportTop && elTop + MIN_EL_HEIGHT_PX < viewportBottom) {
      // this might be a block element, so lets try to find its visible child
      return (
        getFirstVisiblePosInMarkup(viewportEl, el) ?? Number.parseInt(el.dataset.rangeStart!, 10)
      );
    }
  }

  return undefined;
}

function scrollFirstVisiblePosIntoView(markupEl: HTMLElement, pos: number) {
  let minRangeEl: HTMLElement | null = null;
  let minRangeWidth: number | null = null;

  let closestRangeEl: HTMLElement | null = null;
  let closestDistance: number | null = null;

  for (const el of markupEl.querySelectorAll<HTMLElement>('[data-range-start]')) {
    const rangeStart = Number.parseInt(el.dataset.rangeStart!, 10);
    const rangeEnd = Number.parseInt(el.dataset.rangeEnd!, 10);

    const distance = Math.min(Math.abs(rangeStart - pos), Math.abs(rangeEnd - pos));
    if (closestDistance === null || distance < closestDistance) {
      closestRangeEl = el;
      closestDistance = distance;
    }

    if (pos < rangeStart || pos > rangeEnd) {
      continue;
    }

    const rangeWidth = rangeEnd - rangeStart;
    if (!minRangeWidth || rangeWidth < minRangeWidth) {
      minRangeEl = el;
      minRangeWidth = rangeWidth;
    }
  }

  if (minRangeEl) {
    minRangeEl.scrollIntoView({
      block: 'start',
    });
    return;
  }

  if (closestRangeEl) {
    closestRangeEl.scrollIntoView({
      block: 'start',
    });
    return;
  }

  console.warn(`Can't find range that includes pos ${pos}`);
}

type RefContainerProps = {
  id: DocumentId;
  description?: string;
  assetPreview?: boolean;
};
function RefContainer({ id, description, assetPreview }: RefContainerProps) {
  const result = useCachedRef(id);

  if (assetPreview && canPreview(result.documentType, result.data)) {
    return <AssetPreviewBlock documentId={id} data={result.data} description={description} />;
  }

  return (
    <Ref
      documentId={id}
      documentType={result.documentType}
      documentTitle={result.title}
      description={description}
    />
  );
}

export type MarkupRef = {
  getFirstVisiblePos(viewport: HTMLElement): number | undefined;
  scrollToPos(pos: number): void;
};

type MarkupProps = {
  markup: string;
};

export const Markup = forwardRef<MarkupRef, MarkupProps>(function Markup({ markup }, innerRef) {
  const { value } = useSuspenseQuery({ typeName: 'ParseMarkup', markup });

  const markupRef = useRef<HTMLDivElement | null>(null);

  useImperativeHandle(
    innerRef,
    () => ({
      getFirstVisiblePos(viewport) {
        const markupEl = markupRef.current;
        if (!markupEl) {
          throw new Error('Markup el is missing');
        }

        return getFirstVisiblePosInMarkup(viewport, markupEl);
      },
      scrollToPos(pos) {
        const markupEl = markupRef.current;
        if (!markupEl) {
          throw new Error('Markup el is missing');
        }

        scrollFirstVisiblePosIntoView(markupEl, pos);
      },
    }),
    [],
  );

  return markupElementToJSX(value.ast, markupRef);
});
