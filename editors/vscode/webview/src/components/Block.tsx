import { useState } from 'react';
import { AstBlock, BlockType } from '../types';
import { AddElementDropdown } from './AddElementDropdown';
import { generateAddChildEdit, generateDeleteBlockEdit } from '../parser';
import './Block.css';

interface BlockProps {
  block: AstBlock;
  depth: number;
  selectedBlockId: string | null;
  source: string;
  onSelectBlock: (id: string | null) => void;
  onEdit: (edit: { start: number; end: number; text: string }) => void;
}

const TYPE_ICONS: Record<string, string> = {
  app: '📱',
  page: '📄',
  component: '🧩',
  column: '↕️',
  row: '↔️',
  stack: '📚',
  grid: '⊞',
  container: '📦',
  scroll: '📜',
  spacer: '⬜',
  rect: '▢',
  text: 'T',
  heading: 'H',
  image: '🖼️',
  input: '✏️',
  checkbox: '☑️',
  radio: '⭕',
  select: '▼',
  option: '•',
  if: '❓',
  else: '⤵️',
  each: '🔄',
  state: '💾',
  let: '📝',
  data: '📡',
  on: '⚡',
  theme: '🎨',
  use: '🔗',
  slot: '🔲',
  fill: '📥',
};

// Default properties for new elements
const DEFAULT_ELEMENT_PROPS: Record<string, Record<string, { value: string; type: 'string' | 'number' | 'color' | 'boolean' | 'binding' }>> = {
  rect: { width: { value: '100px', type: 'number' }, height: { value: '50px', type: 'number' }, color: { value: '#3b82f6', type: 'color' } },
  text: {},
  heading: {},
  column: { gap: { value: '16px', type: 'number' } },
  row: { gap: { value: '8px', type: 'number' } },
  stack: {},
  grid: { columns: { value: '3', type: 'number' }, gap: { value: '8px', type: 'number' } },
  container: { padding: { value: '16px', type: 'number' } },
  scroll: { height: { value: '400px', type: 'number' } },
  spacer: { height: { value: '16px', type: 'number' } },
  input: { placeholder: { value: 'Enter text', type: 'string' } },
  checkbox: { label: { value: 'Option', type: 'string' } },
  radio: { label: { value: 'Option', type: 'string' } },
  select: {},
  image: { width: { value: '200px', type: 'number' }, height: { value: '150px', type: 'number' } },
};

export function Block({
  block,
  depth,
  selectedBlockId,
  source,
  onSelectBlock,
  onEdit,
}: BlockProps) {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const isSelected = block.id === selectedBlockId;
  const hasChildren = block.children.length > 0;
  const icon = TYPE_ICONS[block.type] || '📦';

  // Handle adding a new child element
  const handleAddElement = (elementType: BlockType) => {
    const defaultProps = DEFAULT_ELEMENT_PROPS[elementType] || {};
    const edit = generateAddChildEdit(block, elementType, source, defaultProps);
    if (edit) {
      onEdit(edit);
    }
  };

  // Handle deleting this block
  const handleDelete = () => {
    const edit = generateDeleteBlockEdit(block, source);
    if (edit) {
      onEdit(edit);
      onSelectBlock(null);
    }
    setShowMenu(false);
  };

  // Format props for display
  const propsDisplay = Object.entries(block.props)
    .slice(0, 3) // Show max 3 props inline
    .map(([key, val]) => {
      const displayVal = val.type === 'color' ? (
        <span className="prop-color" style={{ backgroundColor: val.value }}>
          {val.value}
        </span>
      ) : val.type === 'string' ? (
        `"${val.value.slice(0, 20)}${val.value.length > 20 ? '...' : ''}"`
      ) : (
        val.value
      );
      return (
        <span key={key} className="prop-inline">
          <span className="prop-key">{key}:</span> {displayVal}
        </span>
      );
    });

  const moreProps = Object.keys(block.props).length > 3
    ? `+${Object.keys(block.props).length - 3} more`
    : null;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onSelectBlock(isSelected ? null : block.id);
  };

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (hasChildren) {
      setIsCollapsed(!isCollapsed);
    }
  };

  return (
    <div
      className={`block ${isSelected ? 'selected' : ''} depth-${Math.min(depth, 5)}`}
      data-type={block.type}
    >
      <div
        className="block-header"
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
      >
        {hasChildren && (
          <button
            className="collapse-btn"
            onClick={(e) => {
              e.stopPropagation();
              setIsCollapsed(!isCollapsed);
            }}
          >
            {isCollapsed ? '▶' : '▼'}
          </button>
        )}
        <span className="block-icon">{icon}</span>
        <span className="block-type">{block.type}</span>
        {block.name && <span className="block-name">"{block.name}"</span>}
        <span className="block-props">
          {propsDisplay}
          {moreProps && <span className="props-more">{moreProps}</span>}
        </span>
        <div className="block-menu-container">
          <button
            className="block-menu-btn"
            onClick={(e) => {
              e.stopPropagation();
              setShowMenu(!showMenu);
            }}
          >
            ⋯
          </button>
          {showMenu && (
            <div className="block-menu-dropdown">
              <button className="menu-item" onClick={handleDelete}>
                🗑️ Delete
              </button>
              <button
                className="menu-item"
                onClick={() => {
                  // Duplicate this block
                  // For now just log - full implementation would copy the block
                  console.log('Duplicate:', block.id);
                  setShowMenu(false);
                }}
              >
                📋 Duplicate
              </button>
            </div>
          )}
        </div>
      </div>

      {hasChildren && !isCollapsed && (
        <div className="block-children">
          {block.children.map((child) => (
            <Block
              key={child.id}
              block={child}
              depth={depth + 1}
              selectedBlockId={selectedBlockId}
              source={source}
              onSelectBlock={onSelectBlock}
              onEdit={onEdit}
            />
          ))}
          <AddElementDropdown onAdd={handleAddElement} />
        </div>
      )}

      {/* Show add button for blocks that can have children but currently don't */}
      {!hasChildren && canHaveChildren(block.type) && !isCollapsed && (
        <div className="block-children">
          <AddElementDropdown onAdd={handleAddElement} />
        </div>
      )}
    </div>
  );
}

// Elements that can contain children
function canHaveChildren(type: string): boolean {
  return [
    'app', 'page', 'component', 'column', 'row', 'stack', 'grid',
    'container', 'scroll', 'if', 'each', 'select', 'slot', 'fill'
  ].includes(type);
}
