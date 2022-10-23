const DATE_FORMATTER = new Intl.DateTimeFormat('en-US', {
  hour: 'numeric',
  minute: 'numeric',
  hour12: false,

  day: 'numeric',
  month: 'short',
  year: 'numeric',
});

export function formatDate(date: Date): string {
  return DATE_FORMATTER.format(date);
}

const RELATIVE_DATE_FORMATTER = new Intl.RelativeTimeFormat('en-GB', {
  numeric: 'auto',
});

const SHORT_DATE_FORMATTER = new Intl.DateTimeFormat('en-US', {
  day: 'numeric',
  month: 'short',
  year: 'numeric',
});

export function formatDateHuman(date: Date): string {
  const now = new Date();

  const millis = date.getTime() - now.getTime();
  const seconds = millis / 1000;
  const minutes = seconds / 60;
  const hours = minutes / 60;
  const days = hours / 24;
  const weeks = days / 7;

  if (Math.abs(minutes) < 60) {
    return RELATIVE_DATE_FORMATTER.format(Math.floor(minutes), 'minutes');
  }

  if (Math.abs(hours) < 24) {
    return RELATIVE_DATE_FORMATTER.format(Math.floor(hours), 'hour');
  }

  if (Math.abs(days) < 7) {
    return RELATIVE_DATE_FORMATTER.format(Math.floor(days), 'days');
  }

  if (Math.abs(weeks) < 4) {
    return RELATIVE_DATE_FORMATTER.format(Math.floor(weeks), 'weeks');
  }

  return SHORT_DATE_FORMATTER.format(date);
}
