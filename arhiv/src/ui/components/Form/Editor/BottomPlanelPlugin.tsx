import { EditorView, Panel, showPanel } from '@codemirror/view';

export function createBottomPanel(dom: HTMLElement) {
  return showPanel.of((_view: EditorView): Panel => {
    return {
      dom,
    };
  });
}
