import { AstBlock, BlockType, PropValue } from './types';

/**
 * Simple Naze parser for the visual editor.
 * This is a lightweight parser that extracts structure for visual editing.
 * The authoritative parsing happens in the Rust compiler.
 */

interface ParseResult {
  ast: AstBlock | null;
  error: string | null;
}

let nextId = 0;
function genId(): string {
  return `block-${nextId++}`;
}

export function parseNazeToAst(source: string): ParseResult {
  nextId = 0;
  try {
    const tokens = tokenize(source);
    const ast = parseBlock(tokens, 0);
    return { ast: ast.block, error: null };
  } catch (e) {
    return { ast: null, error: e instanceof Error ? e.message : String(e) };
  }
}

type Token =
  | { type: 'keyword'; value: string; offset: number }
  | { type: 'string'; value: string; offset: number }
  | { type: 'number'; value: string; offset: number }
  | { type: 'color'; value: string; offset: number }
  | { type: 'ident'; value: string; offset: number }
  | { type: 'punct'; value: string; offset: number }
  | { type: 'comment'; value: string; offset: number };

const KEYWORDS = new Set([
  'app', 'page', 'component', 'column', 'row', 'stack', 'grid', 'container',
  'scroll', 'spacer', 'rect', 'text', 'heading', 'image', 'input', 'checkbox',
  'radio', 'select', 'option', 'if', 'else', 'each', 'in', 'state', 'let',
  'data', 'on', 'click', 'change', 'submit', 'set', 'navigate', 'log', 'fetch',
  'true', 'false', 'theme', 'use', 'slot', 'fill', 'link',
]);

function tokenize(source: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;

  while (i < source.length) {
    // Skip whitespace
    if (/\s/.test(source[i])) {
      i++;
      continue;
    }

    // Comment
    if (source[i] === '-' && source[i + 1] === '-') {
      const start = i;
      i += 2;
      while (i < source.length && source[i] !== '\n') i++;
      tokens.push({ type: 'comment', value: source.slice(start, i), offset: start });
      continue;
    }

    // String
    if (source[i] === '"') {
      const start = i;
      i++;
      let value = '';
      while (i < source.length && source[i] !== '"') {
        if (source[i] === '\\' && i + 1 < source.length) {
          value += source[i + 1];
          i += 2;
        } else {
          value += source[i];
          i++;
        }
      }
      i++; // closing quote
      tokens.push({ type: 'string', value, offset: start });
      continue;
    }

    // Color
    if (source[i] === '#') {
      const start = i;
      i++;
      while (i < source.length && /[0-9a-fA-F]/.test(source[i])) i++;
      tokens.push({ type: 'color', value: source.slice(start, i), offset: start });
      continue;
    }

    // Number
    if (/[\d-]/.test(source[i]) && (source[i] !== '-' || /\d/.test(source[i + 1] || ''))) {
      const start = i;
      if (source[i] === '-') i++;
      while (i < source.length && /[\d.]/.test(source[i])) i++;
      // Unit suffix
      if (/[a-z%]/.test(source[i] || '')) {
        while (i < source.length && /[a-z%]/.test(source[i])) i++;
      }
      tokens.push({ type: 'number', value: source.slice(start, i), offset: start });
      continue;
    }

    // Identifier or keyword
    if (/[a-zA-Z_]/.test(source[i])) {
      const start = i;
      while (i < source.length && /[a-zA-Z0-9_-]/.test(source[i])) i++;
      const value = source.slice(start, i);
      const type = KEYWORDS.has(value) ? 'keyword' : 'ident';
      tokens.push({ type, value, offset: start });
      continue;
    }

    // Punctuation
    const punct = source[i];
    tokens.push({ type: 'punct', value: punct, offset: i });
    i++;
  }

  return tokens;
}

interface ParseBlockResult {
  block: AstBlock | null;
  index: number;
}

function parseBlock(tokens: Token[], index: number): ParseBlockResult {
  if (index >= tokens.length) {
    return { block: null, index };
  }

  const token = tokens[index];

  // Skip comments
  if (token.type === 'comment') {
    return parseBlock(tokens, index + 1);
  }

  // Must start with keyword or identifier (for element type)
  if (token.type !== 'keyword' && token.type !== 'ident') {
    return { block: null, index };
  }

  const blockType = token.value as BlockType;
  const block: AstBlock = {
    id: genId(),
    type: blockType,
    props: {},
    children: [],
    span: { offset: token.offset, len: 0 },
  };

  index++;

  // Parse optional name (string after element type)
  if (index < tokens.length && tokens[index].type === 'string') {
    block.name = tokens[index].value;
    index++;
  }

  // Parse properties (key: value pairs before {)
  while (index < tokens.length) {
    const t = tokens[index];

    // Skip comments
    if (t.type === 'comment') {
      index++;
      continue;
    }

    // End of props - start of block body
    if (t.type === 'punct' && t.value === '{') {
      break;
    }

    // End if we hit another element
    if (t.type === 'keyword' && !['true', 'false', 'in'].includes(t.value)) {
      // Check if this is a property name (followed by :)
      if (index + 1 < tokens.length && tokens[index + 1].type === 'punct' && tokens[index + 1].value === ':') {
        // It's a property
      } else {
        break;
      }
    }

    // Property: key: value
    if ((t.type === 'keyword' || t.type === 'ident') &&
        index + 1 < tokens.length &&
        tokens[index + 1].type === 'punct' &&
        tokens[index + 1].value === ':') {
      const propName = t.value;
      index += 2; // skip name and :

      // Parse value
      if (index < tokens.length) {
        const valueToken = tokens[index];
        let propValue: PropValue;

        if (valueToken.type === 'string') {
          propValue = { type: 'string', value: valueToken.value, span: { offset: valueToken.offset, len: valueToken.value.length + 2 } };
        } else if (valueToken.type === 'number') {
          propValue = { type: 'number', value: valueToken.value, span: { offset: valueToken.offset, len: valueToken.value.length } };
        } else if (valueToken.type === 'color') {
          propValue = { type: 'color', value: valueToken.value, span: { offset: valueToken.offset, len: valueToken.value.length } };
        } else if (valueToken.type === 'keyword' && (valueToken.value === 'true' || valueToken.value === 'false')) {
          propValue = { type: 'boolean', value: valueToken.value, span: { offset: valueToken.offset, len: valueToken.value.length } };
        } else if (valueToken.type === 'ident' || valueToken.type === 'keyword') {
          propValue = { type: 'binding', value: valueToken.value, span: { offset: valueToken.offset, len: valueToken.value.length } };
        } else {
          index++;
          continue;
        }

        block.props[propName] = propValue;
        index++;
      }
      continue;
    }

    // Comma separator
    if (t.type === 'punct' && t.value === ',') {
      index++;
      continue;
    }

    // Unknown token in props context
    break;
  }

  // Parse block body if present
  if (index < tokens.length && tokens[index].type === 'punct' && tokens[index].value === '{') {
    index++; // skip {

    while (index < tokens.length) {
      const t = tokens[index];

      // Skip comments
      if (t.type === 'comment') {
        index++;
        continue;
      }

      // End of block
      if (t.type === 'punct' && t.value === '}') {
        index++;
        break;
      }

      // Parse child block
      const childResult = parseBlock(tokens, index);
      if (childResult.block) {
        block.children.push(childResult.block);
        index = childResult.index;
      } else {
        index++;
      }
    }
  }

  // Calculate span length
  if (block.span && index > 0 && index <= tokens.length) {
    const lastToken = tokens[Math.min(index - 1, tokens.length - 1)];
    block.span.len = lastToken.offset + (lastToken.value?.length || 1) - block.span.offset;
  }

  return { block, index };
}

/**
 * Generate Naze source code from an AST block.
 */
export function astToSource(block: AstBlock, indent: number = 0): string {
  const spaces = '  '.repeat(indent);
  let result = spaces + block.type;

  // Add name if present
  if (block.name) {
    result += ` "${block.name}"`;
  }

  // Add properties
  const propEntries = Object.entries(block.props);
  if (propEntries.length > 0) {
    const propStrings = propEntries.map(([key, val]) => {
      let valStr: string;
      switch (val.type) {
        case 'string':
          valStr = `"${val.value}"`;
          break;
        case 'color':
          valStr = val.value.startsWith('#') ? val.value : `#${val.value}`;
          break;
        default:
          valStr = val.value;
      }
      return `${key}: ${valStr}`;
    });
    result += ' ' + propStrings.join(', ');
  }

  // Add children
  if (block.children.length > 0) {
    result += ' {\n';
    for (const child of block.children) {
      result += astToSource(child, indent + 1) + '\n';
    }
    result += spaces + '}';
  }

  return result;
}

// ─── Edit Generation ─────────────────────────────────────────────────────────

export interface SourceEdit {
  start: number;
  end: number;
  text: string;
}

/**
 * Generate an edit to update a property value.
 */
export function generatePropertyEdit(
  block: AstBlock,
  propName: string,
  newValue: string,
  valueType: 'string' | 'number' | 'color' | 'boolean' | 'binding'
): SourceEdit | null {
  const prop = block.props[propName];

  if (prop && prop.span) {
    // Existing property - replace the value
    let formattedValue: string;
    switch (valueType) {
      case 'string':
        formattedValue = `"${newValue}"`;
        break;
      case 'color':
        formattedValue = newValue.startsWith('#') ? newValue : `#${newValue}`;
        break;
      default:
        formattedValue = newValue;
    }

    return {
      start: prop.span.offset,
      end: prop.span.offset + prop.span.len,
      text: formattedValue,
    };
  }

  // Property doesn't exist - need to add it
  // This is more complex as we need to find the right insertion point
  return null;
}

/**
 * Generate an edit to update the block's name (e.g., app "title").
 */
export function generateNameEdit(
  block: AstBlock,
  newName: string,
  source: string
): SourceEdit | null {
  if (!block.span) return null;

  // Find the name position after the element type
  const blockStart = block.span.offset;
  const elementType = block.type;

  // Look for the existing name or the position after element type
  const afterType = blockStart + elementType.length;
  const searchRegion = source.slice(afterType, afterType + 200); // Look in next 200 chars

  const nameMatch = searchRegion.match(/^\s*"([^"]*)"/);
  if (nameMatch) {
    // Replace existing name
    const nameStart = afterType + searchRegion.indexOf('"');
    const nameEnd = nameStart + nameMatch[0].trim().length;
    return {
      start: nameStart,
      end: nameEnd,
      text: `"${newName}"`,
    };
  } else if (block.name) {
    // Name existed but wasn't found (shouldn't happen normally)
    return null;
  } else {
    // Add new name after element type
    return {
      start: afterType,
      end: afterType,
      text: ` "${newName}"`,
    };
  }
}

/**
 * Generate an edit to add a new property to a block.
 */
export function generateAddPropertyEdit(
  block: AstBlock,
  propName: string,
  value: string,
  valueType: 'string' | 'number' | 'color' | 'boolean' | 'binding',
  source: string
): SourceEdit | null {
  if (!block.span) return null;

  let formattedValue: string;
  switch (valueType) {
    case 'string':
      formattedValue = `"${value}"`;
      break;
    case 'color':
      formattedValue = value.startsWith('#') ? value : `#${value}`;
      break;
    default:
      formattedValue = value;
  }

  const propStr = `${propName}: ${formattedValue}`;

  // Find where to insert the property
  // If block has existing props, add after the last one
  // If block has no props, add after the element name/type

  const existingProps = Object.entries(block.props);
  if (existingProps.length > 0) {
    // Find the last property's end position
    let lastPropEnd = 0;
    for (const [, val] of existingProps) {
      if (val.span) {
        const end = val.span.offset + val.span.len;
        if (end > lastPropEnd) {
          lastPropEnd = end;
        }
      }
    }

    if (lastPropEnd > 0) {
      return {
        start: lastPropEnd,
        end: lastPropEnd,
        text: `, ${propStr}`,
      };
    }
  }

  // No existing properties - add after name/type
  const blockStart = block.span.offset;
  const elementType = block.type;
  let insertPos = blockStart + elementType.length;

  // Skip past name if present
  if (block.name) {
    const afterType = source.slice(insertPos, insertPos + 200);
    const nameMatch = afterType.match(/^\s*"[^"]*"/);
    if (nameMatch) {
      insertPos += nameMatch[0].length;
    }
  }

  return {
    start: insertPos,
    end: insertPos,
    text: ` ${propStr}`,
  };
}

/**
 * Generate an edit to add a new child element to a block.
 */
export function generateAddChildEdit(
  parent: AstBlock,
  childType: string,
  source: string,
  defaultProps?: Record<string, { value: string; type: 'string' | 'number' | 'color' | 'boolean' | 'binding' }>
): SourceEdit | null {
  if (!parent.span) return null;

  // Find the closing brace of the parent block
  const blockEnd = parent.span.offset + parent.span.len;

  // Look backwards to find the }
  let closingBrace = -1;
  for (let i = blockEnd - 1; i >= parent.span.offset; i--) {
    if (source[i] === '}') {
      closingBrace = i;
      break;
    }
  }

  if (closingBrace === -1) {
    // Block has no children section - need to add { }
    // Find where props end
    let insertPos = parent.span.offset + parent.type.length;

    // Skip name
    if (parent.name) {
      const afterType = source.slice(insertPos, insertPos + 200);
      const nameMatch = afterType.match(/^\s*"[^"]*"/);
      if (nameMatch) {
        insertPos += nameMatch[0].length;
      }
    }

    // Skip existing props
    const existingProps = Object.entries(parent.props);
    if (existingProps.length > 0) {
      let lastPropEnd = insertPos;
      for (const [, val] of existingProps) {
        if (val.span) {
          const end = val.span.offset + val.span.len;
          if (end > lastPropEnd) {
            lastPropEnd = end;
          }
        }
      }
      insertPos = lastPropEnd;
    }

    // Build the child element
    let childStr = childType;
    if (defaultProps) {
      const propStrs = Object.entries(defaultProps).map(([key, val]) => {
        let formatted: string;
        switch (val.type) {
          case 'string':
            formatted = `"${val.value}"`;
            break;
          case 'color':
            formatted = val.value.startsWith('#') ? val.value : `#${val.value}`;
            break;
          default:
            formatted = val.value;
        }
        return `${key}: ${formatted}`;
      });
      if (propStrs.length > 0) {
        childStr += ' ' + propStrs.join(', ');
      }
    }

    return {
      start: insertPos,
      end: insertPos,
      text: ` {\n  ${childStr}\n}`,
    };
  }

  // Block has children - insert before the closing brace
  // Find the indentation level
  let indent = '  ';
  if (parent.children.length > 0) {
    const firstChild = parent.children[0];
    if (firstChild.span) {
      // Count leading whitespace of first child
      let ws = '';
      for (let i = firstChild.span.offset - 1; i >= 0 && /[ \t]/.test(source[i]); i--) {
        ws = source[i] + ws;
      }
      if (ws) {
        indent = ws;
      }
    }
  }

  // Build the child element
  let childStr = childType;
  if (defaultProps) {
    const propStrs = Object.entries(defaultProps).map(([key, val]) => {
      let formatted: string;
      switch (val.type) {
        case 'string':
          formatted = `"${val.value}"`;
          break;
        case 'color':
          formatted = val.value.startsWith('#') ? val.value : `#${val.value}`;
          break;
        default:
          formatted = val.value;
      }
      return `${key}: ${formatted}`;
    });
    if (propStrs.length > 0) {
      childStr += ' ' + propStrs.join(', ');
    }
  }

  return {
    start: closingBrace,
    end: closingBrace,
    text: `${indent}${childStr}\n`,
  };
}

/**
 * Generate an edit to delete a block.
 */
export function generateDeleteBlockEdit(
  block: AstBlock,
  source: string
): SourceEdit | null {
  if (!block.span) return null;

  // Find the start of the line (to remove indentation too)
  let lineStart = block.span.offset;
  while (lineStart > 0 && source[lineStart - 1] !== '\n') {
    lineStart--;
  }

  // Find the end of the block including the newline
  let blockEnd = block.span.offset + block.span.len;
  if (source[blockEnd] === '\n') {
    blockEnd++;
  }

  return {
    start: lineStart,
    end: blockEnd,
    text: '',
  };
}
