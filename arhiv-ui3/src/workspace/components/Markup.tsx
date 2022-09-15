import { createElement } from 'preact';
import { MarkupElement, throwBadMarkupElement } from '../dto';
import { useQuery } from '../hooks';
import { JSXElement } from '../jsx';
import { RPC } from '../rpc';
import { QueryError } from './QueryError';

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
        },
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
    case 'Link': {
      // TODO handle link_type?
      return (
        <a
          data-range-start={el.range.start}
          data-range-end={el.range.end}
          href={el.url}
          title={el.title}
          target="_blank"
          rel="noopen noreferer"
        >
          {el.children.map(markupElementToJSX)}
        </a>
      );
    }
    case 'Image': {
      // TODO handle link_type?
      return (
        <img
          data-range-start={el.range.start}
          data-range-end={el.range.end}
          href={el.url}
          title={el.title}
          alt={String(el.children.map(markupElementToJSX))}
        />
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
