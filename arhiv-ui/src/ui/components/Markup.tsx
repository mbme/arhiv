import { createElement } from 'preact';
import { cx, Obj } from 'utils';
import { DocumentId, MarkupElement, throwBadMarkupElement } from 'dto';
import { useSuspense } from 'utils/hooks';
import { JSXElement } from 'utils/jsx';
import { RPC } from 'utils/rpc';
import { Link } from 'components/Link';
import { RefContainer } from 'components/Ref';

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

function markupElementToJSX(el: MarkupElement): JSXElement {
  switch (el.typeName) {
    case 'Document': {
      return (
        <div className="markup w-full">{el.children.map((child) => markupElementToJSX(child))}</div>
      );
    }
    case 'Text': {
      return (
        <span data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.value}
        </span>
      );
    }
    case 'Code': {
      return (
        <code data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.value}
        </code>
      );
    }
    case 'Html': {
      return (
        <span
          data-range-start={el.range.start}
          data-range-end={el.range.end}
          dangerouslySetInnerHTML={{ __html: el.value }}
        />
      );
    }
    case 'SoftBreak': {
      return <br data-range-start={el.range.start} data-range-end={el.range.end} />;
    }
    case 'HardBreak': {
      return <br data-range-start={el.range.start} data-range-end={el.range.end} />;
    }
    case 'Rule': {
      return <hr data-range-start={el.range.start} data-range-end={el.range.end} />;
    }
    case 'Paragraph': {
      return (
        <p data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </p>
      );
    }
    case 'Heading': {
      return createElement(
        el.level,
        {
          'data-range-start': el.range.start,
          'data-range-end': el.range.end,
        } as Obj<unknown>,
        ...el.children.map((child) => markupElementToJSX(child))
      );
    }
    case 'BlockQuote': {
      return (
        <blockquote data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </blockquote>
      );
    }
    case 'CodeBlock': {
      // TODO handle kind
      return (
        <pre>
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
          'start': el.first_item_number ?? undefined,
          'data-range-start': el.range.start,
          'data-range-end': el.range.end,
        },

        ...el.children.map((child) => markupElementToJSX(child))
      );
    }
    case 'TaskListMarker': {
      return (
        <input
          type="checkbox"
          className="mr-1"
          checked={el.checked}
          disabled
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        />
      );
    }
    case 'ListItem': {
      return (
        <li data-range-start={el.range.start} data-range-end={el.range.end}>
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
      return (
        <table data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </table>
      );
    }
    case 'TableHead': {
      return (
        <thead>
          <tr data-range-start={el.range.start} data-range-end={el.range.end}>
            {el.children.map((col) => {
              if (col.typeName !== 'TableCell') {
                throw new Error(`Expected TableCell, got ${col.typeName}`);
              }

              return <th>{col.children.map((child) => markupElementToJSX(child))}</th>;
            })}
          </tr>
        </thead>
      );
    }
    case 'TableRow': {
      return (
        <tr data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </tr>
      );
    }
    case 'TableCell': {
      return (
        <td data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </td>
      );
    }
    case 'Emphasis': {
      return (
        <em data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </em>
      );
    }
    case 'Strong': {
      return (
        <strong data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </strong>
      );
    }
    case 'Strikethrough': {
      return (
        <s data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map((child) => markupElementToJSX(child))}
        </s>
      );
    }
    case 'Link':
    case 'Image': {
      let description = extractText(el.children);

      if (el.url.startsWith('ref:')) {
        const id = el.url.substring('ref:'.length) as DocumentId;
        const preview = el.typeName === 'Image';

        if (el.link_type === 'Autolink') {
          description = '';
        }

        return (
          <span
            data-range-start={el.range.start}
            data-range-end={el.range.end}
            className={cx({
              'inline-block w-full': el.typeName === 'Image',
            })}
          >
            <RefContainer id={id} attachmentPreview={preview} description={description} />
          </span>
        );
      }

      // TODO handle link_type?
      return (
        <span data-range-start={el.range.start} data-range-end={el.range.end}>
          <Link url={el.url}>{description}</Link>
        </span>
      );
    }
  }

  throwBadMarkupElement(el);
}

type MarkupProps = {
  markup: string;
};

export function Markup({ markup }: MarkupProps) {
  const result = useSuspense(markup, () => RPC.ParseMarkup({ markup }), [markup]);

  return markupElementToJSX(result.ast);
}
