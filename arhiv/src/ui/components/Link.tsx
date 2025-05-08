import { cx } from 'utils';
import { JSXChildren } from 'utils/jsx';
import { Icon } from 'components/Icon';

type LinkProps = {
  url: string;
  title?: string;
  children: JSXChildren;
};
export function Link({ url, title, children }: LinkProps) {
  return (
    <a href={url} title={title ?? url} target="_blank" rel="noreferrer" className="var-link-color">
      {children}
    </a>
  );
}

type DownloadLinkProps = {
  url: string;
  fileName: string;
  title: string;
  className?: string;
};
export function DownloadLink({ url, fileName, title, className }: DownloadLinkProps) {
  return (
    <a
      href={url}
      title={fileName}
      download={fileName}
      target="_blank"
      rel="noreferrer"
      className={cx(
        'flex items-center gap-2 var-active-color hover:var-active-color-hover',
        className,
      )}
    >
      <Icon variant="download" />

      {title}
    </a>
  );
}
