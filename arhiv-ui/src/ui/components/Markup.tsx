import { createElement } from 'react';
import { cx, Obj } from 'utils';
import { DocumentId, MarkupElement, throwBadMarkupElement, Range } from 'dto';
import { useSuspense } from 'utils/suspense';
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

function rangeToString(range: Range): string {
  return `${range.start}-${range.end}`;
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
        ...el.children.map((child) => markupElementToJSX(child))
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

        ...el.children.map((child) => markupElementToJSX(child))
      );
    }
    case 'TaskListMarker': {
      return (
        <input
          key={rangeToString(el.range)}
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
      return (
        <table
          key={rangeToString(el.range)}
          data-range-start={el.range.start}
          data-range-end={el.range.end}
        >
          {el.children.map((child) => markupElementToJSX(child))}
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

      if (el.url.startsWith('ref:')) {
        const id = el.url.substring('ref:'.length) as DocumentId;
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
            <RefContainer id={id} attachmentPreview={preview} description={description} />
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

type MarkupProps = {
  markup: string;
};

export function Markup({ markup }: MarkupProps) {
  const { value } = useSuspense(markup, () => RPC.ParseMarkup({ markup }));

  return markupElementToJSX(value.ast);
}
