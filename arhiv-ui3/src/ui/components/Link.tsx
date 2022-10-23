type Props = {
  url: string;
  title?: string;
  description: string;
};
export function Link({ url, title, description }: Props) {
  return (
    <a
      href={url}
      title={title}
      target="_blank"
      rel="noopen noreferer"
      className="text-orange-600/70 hover:text-orange-700/100 transition-colors"
    >
      {description}
    </a>
  );
}
