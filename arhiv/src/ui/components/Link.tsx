import { JSXChildren } from 'utils/jsx';

type Props = {
  url: string;
  title?: string;
  children: JSXChildren;
};
export function Link({ url, title, children }: Props) {
  return (
    <a
      href={url}
      title={title ?? url}
      target="_blank"
      rel="noreferrer"
      className="text-orange-600/70 hover:text-orange-700/100 transition-colors"
    >
      {children}
    </a>
  );
}
