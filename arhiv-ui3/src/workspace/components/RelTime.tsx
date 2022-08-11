import { formatDate, formatDateHuman } from '../../scripts/date';

type RelTimeProps = {
  datetime: string;
  className?: string;
};

export function RelTime({ datetime, className }: RelTimeProps) {
  const date = new Date(datetime);

  return (
    <time dateTime={datetime} title={formatDate(date)} className={className}>
      {formatDateHuman(date)}
    </time>
  );
}
