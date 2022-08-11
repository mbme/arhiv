import { formatDate, formatDateHuman } from '../../scripts/date';

type RelTimeProps = {
  datetime: string;
  className?: string;
  relative?: boolean;
};

export function DateTime({ datetime, className, relative }: RelTimeProps) {
  const date = new Date(datetime);

  const dateStr = formatDate(date);

  return (
    <time dateTime={datetime} title={relative ? dateStr : undefined} className={className}>
      {relative ? formatDateHuman(date) : dateStr}
    </time>
  );
}
