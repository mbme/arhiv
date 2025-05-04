import { Icon } from 'components/Icon';

export const CONFLICT_INDICATOR = (
  <span title="This document has conflict">
    <Icon variant="error-triangle" className="text-red-700" />
  </span>
);

export const STAGED_INDICATOR = (
  <span title="This document is modified">
    <Icon variant="pencil" className="text-lime-600 size-4" />
  </span>
);
