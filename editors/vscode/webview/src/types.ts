// VS Code API type
declare global {
  interface Window {
    acquireVsCodeApi: () => VsCodeApi;
  }
}

export interface VsCodeApi {
  postMessage(message: unknown): void;
  getState(): unknown;
  setState(state: unknown): void;
}

// AST types matching naze-parser
export type BlockType =
  | 'app'
  | 'page'
  | 'component'
  | 'column'
  | 'row'
  | 'stack'
  | 'grid'
  | 'container'
  | 'scroll'
  | 'spacer'
  | 'rect'
  | 'text'
  | 'heading'
  | 'image'
  | 'input'
  | 'checkbox'
  | 'radio'
  | 'select'
  | 'option'
  | 'if'
  | 'else'
  | 'each'
  | 'state'
  | 'let'
  | 'data'
  | 'on'
  | 'theme'
  | 'use'
  | 'slot'
  | 'fill';

export interface PropValue {
  type: 'string' | 'number' | 'color' | 'boolean' | 'expression' | 'binding';
  value: string;
  span?: { offset: number; len: number };
}

export interface AstBlock {
  id: string;
  type: BlockType;
  name?: string; // For app, page, component, state, etc.
  props: Record<string, PropValue>;
  children: AstBlock[];
  span?: { offset: number; len: number };
}

export interface DocumentState {
  fileName: string;
  content: string;
  ast: AstBlock | null;
  parseError: string | null;
}

// Messages between extension and webview
export type ExtensionMessage =
  | { type: 'update'; content: string; fileName: string }
  | { type: 'ai-response'; content?: string; error?: string };

export type WebviewMessage =
  | { type: 'ready' }
  | { type: 'edit'; edit: { start: number; end: number; text: string } }
  | { type: 'ai-request'; prompt: string; context?: string };

// Element categories for the add dropdown
export const ELEMENT_CATEGORIES = {
  layout: {
    label: 'Layout',
    icon: '📐',
    elements: ['column', 'row', 'stack', 'grid', 'container', 'scroll', 'spacer'],
  },
  content: {
    label: 'Content',
    icon: '📝',
    elements: ['text', 'heading', 'image', 'rect'],
  },
  form: {
    label: 'Form',
    icon: '📋',
    elements: ['input', 'checkbox', 'radio', 'select'],
  },
  logic: {
    label: 'Logic',
    icon: '🔀',
    elements: ['if', 'each', 'state', 'let'],
  },
} as const;

// Element metadata
export const ELEMENT_INFO: Record<string, { description: string; props: string[] }> = {
  app: { description: 'Root application container', props: ['title'] },
  page: { description: 'A navigable page', props: ['path'] },
  component: { description: 'Reusable component definition', props: [] },
  column: { description: 'Vertical flex container', props: ['gap', 'padding', 'align', 'justify'] },
  row: { description: 'Horizontal flex container', props: ['gap', 'padding', 'align', 'justify', 'wrap'] },
  stack: { description: 'Layered container (z-axis)', props: ['padding'] },
  grid: { description: 'CSS grid container', props: ['columns', 'gap', 'padding'] },
  container: { description: 'Generic container', props: ['width', 'height', 'padding'] },
  scroll: { description: 'Scrollable container', props: ['overflow'] },
  spacer: { description: 'Flexible space', props: ['width', 'height'] },
  rect: { description: 'Rectangle shape', props: ['width', 'height', 'color', 'radius', 'border', 'opacity'] },
  text: { description: 'Text content', props: ['color', 'font-size', 'align'] },
  heading: { description: 'Heading text', props: ['color', 'font-size', 'align'] },
  image: { description: 'Image element', props: ['src', 'alt', 'width', 'height', 'fit'] },
  input: { description: 'Text input field', props: ['bind', 'placeholder', 'type', 'validate'] },
  checkbox: { description: 'Checkbox input', props: ['bind', 'label'] },
  radio: { description: 'Radio button', props: ['bind', 'label'] },
  select: { description: 'Dropdown select', props: ['bind'] },
  option: { description: 'Select option', props: [] },
  if: { description: 'Conditional rendering', props: [] },
  else: { description: 'Else branch', props: [] },
  each: { description: 'Iteration over collection', props: ['in'] },
  state: { description: 'Reactive state variable', props: [] },
  let: { description: 'Local variable binding', props: [] },
  data: { description: 'Data fetch definition', props: ['url', 'method'] },
  on: { description: 'Event handler', props: [] },
  theme: { description: 'Theme definition', props: [] },
  use: { description: 'Theme usage', props: [] },
  slot: { description: 'Component slot', props: ['name'] },
  fill: { description: 'Fill a slot', props: ['slot'] },
};
