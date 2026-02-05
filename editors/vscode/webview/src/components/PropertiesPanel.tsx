import { useCallback } from 'react';
import { AstBlock, ELEMENT_INFO } from '../types';
import { generatePropertyEdit, generateAddPropertyEdit, generateNameEdit } from '../parser';
import './PropertiesPanel.css';

interface PropertiesPanelProps {
  block: AstBlock;
  source: string;
  onEdit: (edit: { start: number; end: number; text: string }) => void;
  onClose: () => void;
}

export function PropertiesPanel({ block, source, onEdit, onClose }: PropertiesPanelProps) {
  const elementInfo = ELEMENT_INFO[block.type];
  const availableProps = elementInfo?.props || [];

  // Handle property value change
  const handlePropertyChange = useCallback((propName: string, newValue: string, valueType: 'string' | 'number' | 'color' | 'boolean' | 'binding') => {
    // First try to generate an edit for existing property
    const edit = generatePropertyEdit(block, propName, newValue, valueType);
    if (edit) {
      onEdit(edit);
      return;
    }

    // Property doesn't exist - add it
    const addEdit = generateAddPropertyEdit(block, propName, newValue, valueType, source);
    if (addEdit) {
      onEdit(addEdit);
    }
  }, [block, source, onEdit]);

  // Handle name change
  const handleNameChange = useCallback((newName: string) => {
    const edit = generateNameEdit(block, newName, source);
    if (edit) {
      onEdit(edit);
    }
  }, [block, source, onEdit]);

  // Group properties
  const dimensionProps = ['width', 'height', 'min-width', 'max-width', 'min-height', 'max-height'];
  const appearanceProps = ['color', 'radius', 'opacity', 'border'];
  const layoutProps = ['padding', 'gap', 'align', 'justify', 'flex-grow', 'flex-shrink', 'wrap', 'columns', 'overflow'];
  const accessibilityProps = ['role', 'label', 'id', 'tab-index'];
  const contentProps = ['src', 'alt', 'fit', 'bind', 'placeholder', 'type', 'validate', 'to'];

  const categorizedProps = [
    { name: 'Dimensions', props: dimensionProps.filter(p => availableProps.includes(p) || block.props[p]) },
    { name: 'Appearance', props: appearanceProps.filter(p => availableProps.includes(p) || block.props[p]) },
    { name: 'Layout', props: layoutProps.filter(p => availableProps.includes(p) || block.props[p]) },
    { name: 'Content', props: contentProps.filter(p => availableProps.includes(p) || block.props[p]) },
    { name: 'Accessibility', props: accessibilityProps.filter(p => availableProps.includes(p) || block.props[p]) },
  ].filter(cat => cat.props.length > 0);

  return (
    <div className="properties-panel">
      <div className="panel-header">
        <span className="panel-title">{block.type} Properties</span>
        <button className="panel-close" onClick={onClose}>×</button>
      </div>

      <div className="panel-content">
        {block.name !== undefined && (
          <div className="prop-group">
            <div className="prop-group-header">Identity</div>
            <div className="prop-row">
              <label className="prop-label">Name</label>
              <input
                className="prop-input"
                type="text"
                defaultValue={block.name}
                onBlur={(e) => handleNameChange(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    handleNameChange((e.target as HTMLInputElement).value);
                  }
                }}
              />
            </div>
          </div>
        )}

        {categorizedProps.map(category => (
          <div key={category.name} className="prop-group">
            <div className="prop-group-header">{category.name}</div>
            {category.props.map(propName => {
              const propValue = block.props[propName];
              return (
                <PropertyInput
                  key={propName}
                  name={propName}
                  value={propValue}
                  onChange={(newValue, valueType) => {
                    handlePropertyChange(propName, newValue, valueType);
                  }}
                />
              );
            })}
          </div>
        ))}

        {/* Event handlers section */}
        <div className="prop-group">
          <div className="prop-group-header">Events</div>
          <button className="add-event-btn">
            + Add event handler
          </button>
          {block.children
            .filter(child => child.type === 'on')
            .map(handler => (
              <div key={handler.id} className="event-item">
                <span className="event-icon">⚡</span>
                <span className="event-name">
                  on {handler.name || 'click'}
                </span>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
}

interface PropertyInputProps {
  name: string;
  value?: { type: string; value: string };
  onChange: (value: string, valueType: 'string' | 'number' | 'color' | 'boolean' | 'binding') => void;
}

function PropertyInput({ name, value, onChange }: PropertyInputProps) {
  const inputValue = value?.value || '';
  const inputType = value?.type || 'string';

  // Determine the value type for this property
  const getValueType = (propName: string, val: string): 'string' | 'number' | 'color' | 'boolean' | 'binding' => {
    if (propName === 'color' || propName === 'border-color' || val.startsWith('#')) {
      return 'color';
    }
    if (['width', 'height', 'padding', 'gap', 'radius', 'font-size', 'opacity', 'border',
         'min-width', 'max-width', 'min-height', 'max-height', 'flex-grow', 'flex-shrink', 'columns'].includes(propName)) {
      return 'number';
    }
    if (['wrap'].includes(propName) || val === 'true' || val === 'false') {
      return 'boolean';
    }
    if (propName === 'bind') {
      return 'binding';
    }
    return 'string';
  };

  // Color picker for color props
  if (name === 'color' || name === 'border-color' || inputType === 'color') {
    return (
      <div className="prop-row">
        <label className="prop-label">{name}</label>
        <div className="color-input-wrapper">
          <input
            type="color"
            className="color-picker"
            defaultValue={inputValue.startsWith('#') ? inputValue : '#ffffff'}
            onChange={(e) => onChange(e.target.value, 'color')}
          />
          <input
            type="text"
            className="prop-input color-text"
            defaultValue={inputValue}
            onBlur={(e) => onChange(e.target.value, 'color')}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                onChange((e.target as HTMLInputElement).value, 'color');
              }
            }}
            placeholder="#000000"
          />
        </div>
      </div>
    );
  }

  // Dropdown for enum props
  const enumValues: Record<string, string[]> = {
    align: ['start', 'center', 'end', 'stretch'],
    justify: ['start', 'center', 'end', 'space-between', 'space-around'],
    overflow: ['visible', 'hidden', 'scroll', 'auto'],
    fit: ['contain', 'cover', 'fill', 'none'],
    type: ['text', 'email', 'password', 'number', 'tel', 'url'],
    role: ['button', 'link', 'checkbox', 'radio', 'slider', 'tab'],
  };

  if (enumValues[name]) {
    return (
      <div className="prop-row">
        <label className="prop-label">{name}</label>
        <select
          className="prop-select"
          defaultValue={inputValue}
          onChange={(e) => onChange(e.target.value, 'string')}
        >
          <option value="">--</option>
          {enumValues[name].map(opt => (
            <option key={opt} value={opt}>{opt}</option>
          ))}
        </select>
      </div>
    );
  }

  // Default text input
  const valueType = getValueType(name, inputValue);

  return (
    <div className="prop-row">
      <label className="prop-label">{name}</label>
      <input
        type="text"
        className="prop-input"
        defaultValue={inputValue}
        onBlur={(e) => onChange(e.target.value, valueType)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            onChange((e.target as HTMLInputElement).value, valueType);
          }
        }}
        placeholder={name === 'opacity' ? '1.0' : ''}
      />
    </div>
  );
}
