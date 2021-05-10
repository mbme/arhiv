import { theme } from './theme'

export const globalStyles = `
/* based on reboot.css */

:root {
  --font-size: 16px;
  --line-height: 1.40;

  --font-family-base: -apple-system, BlinkMacSystemFont, /* Safari Mac/iOS, Chrome */
      "Segoe UI", Roboto, Oxygen, /* Windows, Android, KDE */
      Ubuntu, Cantarell, "Fira Sans", /* Ubuntu, Gnome, Firefox OS */
      "Droid Sans", "Helvetica Neue", sans-serif; /* Old Android */

  --font-family-serif: ui-serif, Georgia, Cambria, "Times New Roman", Times, serif;

  --font-family-mono: SFMono-Regular,
      Menlo,
      Monaco,
      Consolas,
      "Liberation Mono",
      "Courier New",
      monospace;

  --color-primary: #FF553C;
  --color-secondary: #5E5A57;
  --color-text: #333333;
  --color-text-light: #ffffff;
  --color-heading: #000000;
  --color-link: #FDAF3C;
  --color-bg0: #ffffff;
  --color-bg-overlay: rgba(255,255,255, .65);

  --color-bg-primary: #FF553Cd1;
  --color-bg-secondary: #5e5a5759;

  --border-radius-form: 2px;
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-primary: #88c0d0;
    --color-secondary: #8fbcbb;
    --color-text: #d8dee9;
    --color-text-light: #eceff4;
    --color-heading: #d8dee9;
    --color-link: #81a1c1;
    --color-bg0: #2e3440;
    --color-bg-overlay: #4c566a;
  }
}

@media screen and (min-width: 768px) {
  :root {
    --font-size: 18px;
    --line-height: 1.60;
  }
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

:focus {
  outline: none;
  border: 1px solid var(--color-primary) !important;
}

html {
  font-family: var(--font-family-base);
  font-size: var(--font-size);
  line-height: var(--line-height);
  font-variant-numeric: tabular-nums;
  height: 100%;
  width: 100%;

  overflow: hidden;
}

/* Body */
/* 1. Remove the margin in all browsers. */
/* 2. As a best practice, apply a default background - color. */
/* 3. Set an explicit initial text-align value so that we can
      later use the inherit value on things like <th> elements. */

body {
  margin: 0; /* 1 */
  padding: 0;
  font-family: var(--font-family-base);
  font-size: var(--font-size);
  line-height: var(--line-height);
  font-weight: 400;
  text-align: left; /* 3 */
  text-rendering: optimizeLegibility;

  background-color: var(--color-bg0); /* 2 */
  color: var(--color-text);

  height: 100%;
  width: 100%;

  overflow: auto;
}

/* Suppress the focus outline on elements that cannot be accessed via keyboard. */
/* This prevents an unwanted focus outline from appearing around elements that */
/* might still respond to pointer events. */
[tabindex="-1"]:focus {
  outline: none !important;
}


/* Content grouping */
/* 1. Add the correct box sizing in Firefox. */
/* 2. Show the overflow in Edge and IE. */
hr {
  box-sizing: content-box; /* 1 */
  height: 0; /* 1 */
  overflow: visible; /* 2 */
}

/* Typography */
/* Remove top margins from headings */

/* By default, <h1>-<h6> all receive top and bottom margins. We nuke the top */
/* margin for easier control within type scales as it avoids margin collapsing. */
h1, h2, h3, h4, h5, h6 {
  margin-top: 0;
  margin-bottom: ${theme.spacing.medium};
  color: var(--color-heading);
  font-family: var(--font-family-serif);
}

/* Reset margins on paragraphs */
/* Similarly, the top margin on <p>s get reset. However, we also reset the */
/* bottom margin to use rem units instead of em. */
p {
  margin-top: 0;
  margin-bottom: ${theme.spacing.medium};
}

/* Abbreviations */
/* 2. Add the correct text decoration in Chrome, Edge, IE, Opera, and Safari. */
/* 3. Add explicit cursor to indicate changed behavior. */
abbr[title] {
  text-decoration: underline; /* 2 */
  text-decoration: underline dotted; /* 2 */
  cursor: help; /* 3 */
}

address {
  margin-bottom: ${theme.spacing.medium};
  font-style: normal;
  line-height: inherit;
}

ol,
ul,
dl {
  margin-top: 0;
  margin-bottom: ${theme.spacing.medium};
}

ol ol,
ul ul,
ol ul,
ul ol {
  margin-bottom: 0;
}

dt {
  font-weight: 700;
}

dd {
  margin-bottom: .5rem;
  margin-left: 0; /* Undo browser default */
}

blockquote {
  margin: 0 0 ${theme.spacing.medium};
}

b,
strong {
  font-weight: bolder; /* Add the correct font weight in Chrome, Edge, and Safari */
}

small {
  font-size: ${theme.fontSize.fine};
  color: var(--color-secondary);
}

/* Prevent sub and sup elements from affecting the line height in all browsers. */
sub,
sup {
  position: relative;
  font-size: 75%;
  line-height: 0;
  vertical-align: baseline;
}

sub { bottom: -.25em; }
sup { top: -.5em; }


/* Links */
a {
  color: var(--color-link);
  text-decoration: none;
}
a:hover {
  color: var(--color-link);
  text-decoration: none;
}

/* Code */
pre,
code,
kbd,
samp {
  font-family: var(--font-family-mono); /* Correct the inheritance and scaling of font size in all browsers. */
  font-size: 1em; /* Correct the odd em font sizing in all browsers. */
}

pre {
  margin-top: 0; /* Remove browser default top margin */
  margin-bottom: ${theme.spacing.medium};
  overflow: auto; /* Don't allow content to break outside */
}


figure {
  margin: 0 0 ${theme.spacing.medium};
}

/* Images and content */
img {
  width: auto;
  max-width: 100%;
  display: block;
  margin: 0 auto;
}

svg:not(:root) {
  overflow: hidden; /* Hide the overflow in IE */
}

/* Forms */
label {
  /* Allow labels to use margin for spacing. */
  display: inline-block;
  margin-bottom: .5rem;
}

input,
button,
select,
optgroup,
textarea {
  margin: 0; /* Remove the margin in Firefox and Safari */
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  color: inherit;
}

button,
input {
  overflow: visible; /* Show the overflow in Edge */
}

button,
select {
  text-transform: none; /* Remove the inheritance of text transform in Firefox */
}

/* Remove inner border and padding from Firefox, but don't restore the outline like Normalize. */
button::-moz-focus-inner,
[type="button"]::-moz-focus-inner,
[type="reset"]::-moz-focus-inner,
[type="submit"]::-moz-focus-inner {
  padding: 0;
  border-style: none;
}

textarea {
  overflow: auto; /* Remove the default vertical scrollbar in IE. */
  /* Textareas should really only resize vertically so they don't break their (horizontal) containers. */
  resize: vertical;
}

fieldset {
  /* Browsers set a default min - width: min - content;  on fieldsets, */
  /* unlike e.g. <div>s, which have min - width: 0;  by default. */
  /* So we reset that to ensure fieldsets behave more like a standard block element. */
  min-width: 0;
  /* Reset the default outline behavior of fieldsets so they don't affect page layout. */
  padding: 0;
  margin: 0;
  border: 0;
}

/* 1. Correct the text wrapping in Edge and IE. */
/* 2. Correct the color inheritance from fieldset elements in IE. */
legend {
  display: block;
  width: 100%;
  max-width: 100%; /* 1 */
  padding: 0;
  margin-bottom: .5rem;
  font-size: 1.5rem;
  line-height: inherit;
  color: inherit; /* 2 */
  white-space: normal; /* 1 */
}

progress {
  vertical-align: baseline; /* Add the correct vertical alignment in Chrome, Firefox, and Opera. */
}

/* Correct element displays */

output {
  display: inline-block;
}

summary {
  display: list-item; /* Add the correct display in all browsers */
}

select {
  cursor: pointer;
  background: inherit;
  border: 0 none;
}

/* Tables */
table {
  text-align: justify;
  width: 100%;
  border-collapse: collapse;
}

td, th {
  padding: 0.5em;
  border-bottom: 1px solid #f1f1f1;
}

blockquote {
  margin-left: 0px;
  margin-right: 0px;
  padding-left: 1em;
  padding-top: 0.8em;
  padding-bottom: 0.8em;
  padding-right: 0.8em;
  border-left: 5px solid #1d7484;
  margin-bottom: 2.5rem;
  background-color: #f1f1f1;
}

blockquote p {
  margin-bottom: 0;
}

/* Pre and Code */
pre {
  background-color: #f1f1f1;
  display: block;
  padding: 1em;
  overflow-x: auto;
  margin-top: 0px;
  margin-bottom: 2.5rem;
}

code {
  font-size: 0.9em;
  padding: 0 0.5em;
  background-color: #f1f1f1;
  white-space: pre-wrap;
}

pre > code {
  padding: 0;
  background-color: transparent;
  white-space: pre;
}

/* Always hide an element with the hidden HTML attribute (from PureCSS). */
[hidden] {
  display: none !important;
}
`
