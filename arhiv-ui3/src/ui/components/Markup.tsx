import { createElement } from 'preact';
import { Obj } from '../utils';
import { MarkupElement, throwBadMarkupElement } from '../../dto';
import { useQuery } from '../hooks';
import { JSXElement } from '../jsx';
import { RPC } from '../rpc';
import { Link } from './Link';
import { QueryError } from './QueryError';
import { RefContainer } from './Ref';

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
      return <div className="markup">{el.children.map(markupElementToJSX)}</div>;
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
          {el.children.map(markupElementToJSX)}
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
        ...el.children.map(markupElementToJSX)
      );
    }
    case 'BlockQuote': {
      return (
        <blockquote data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map(markupElementToJSX)}
        </blockquote>
      );
    }
    case 'CodeBlock': {
      // TODO handle kind
      return (
        <pre>
          <code data-range-start={el.range.start} data-range-end={el.range.end}>
            {el.children.map(markupElementToJSX)}
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

        ...el.children.map(markupElementToJSX)
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
          {el.children.map(markupElementToJSX)}
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
          {el.children.map(markupElementToJSX)}
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

              return <th>{col.children.map(markupElementToJSX)}</th>;
            })}
          </tr>
        </thead>
      );
    }
    case 'TableRow': {
      return (
        <tr data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map(markupElementToJSX)}
        </tr>
      );
    }
    case 'TableCell': {
      return (
        <td data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map(markupElementToJSX)}
        </td>
      );
    }
    case 'Emphasis': {
      return (
        <em data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map(markupElementToJSX)}
        </em>
      );
    }
    case 'Strong': {
      return (
        <strong data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map(markupElementToJSX)}
        </strong>
      );
    }
    case 'Strikethrough': {
      return (
        <s data-range-start={el.range.start} data-range-end={el.range.end}>
          {el.children.map(markupElementToJSX)}
        </s>
      );
    }
    case 'Link':
    case 'Image': {
      let description = extractText(el.children);

      if (el.link_type === 'Autolink') {
        description = '';
      }

      if (el.url.startsWith('ref:')) {
        const id = el.url.substring('ref:'.length);
        const preview = el.typeName === 'Image';

        return (
          <span data-range-start={el.range.start} data-range-end={el.range.end}>
            <RefContainer
              id={id}
              attachmentPreview={preview}
              title={el.title}
              description={description}
            />
          </span>
        );
      }

      // TODO handle link_type?
      return (
        <span data-range-start={el.range.start} data-range-end={el.range.end}>
          <Link url={el.url} title={el.title} description={description} />
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
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ParseMarkup({ markup }, abortSignal),
    {
      refreshIfChange: [markup],
    }
  );

  if (error) {
    return <QueryError error={error} />;
  }

  if (inProgress || !result) {
    return null;
  }

  return markupElementToJSX(result.ast);
}
