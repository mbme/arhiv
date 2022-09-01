// Prevent implicit submission of the form
export function PreventImplicitSubmissionOnEnter() {
  return <button type="submit" disabled style="display: none" hidden aria-hidden="true"></button>;
}
