import { useState, useRef, useEffect } from 'react';
import { ELEMENT_CATEGORIES, ELEMENT_INFO, BlockType } from '../types';
import './AddElementDropdown.css';

interface AddElementDropdownProps {
  onAdd: (elementType: BlockType) => void;
}

export function AddElementDropdown({ onAdd }: AddElementDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      inputRef.current?.focus();
    }

    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  // Filter elements based on search
  const filteredCategories = Object.entries(ELEMENT_CATEGORIES).map(([key, category]) => {
    const elements = category.elements.filter(el =>
      el.toLowerCase().includes(searchTerm.toLowerCase()) ||
      ELEMENT_INFO[el]?.description.toLowerCase().includes(searchTerm.toLowerCase())
    );
    return { key, ...category, elements };
  }).filter(cat => cat.elements.length > 0);

  const handleSelect = (elementType: string) => {
    onAdd(elementType as BlockType);
    setIsOpen(false);
    setSearchTerm('');
  };

  return (
    <div className="add-element-dropdown" ref={dropdownRef}>
      <button
        className="add-element-btn"
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className="add-icon">+</span>
        <span>Add element...</span>
      </button>

      {isOpen && (
        <div className="dropdown-menu">
          <input
            ref={inputRef}
            type="text"
            className="dropdown-search"
            placeholder="Search elements..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Escape') {
                setIsOpen(false);
              }
            }}
          />

          <div className="dropdown-categories">
            {filteredCategories.map(category => (
              <div key={category.key} className="dropdown-category">
                <div className="category-header">
                  <span className="category-icon">{category.icon}</span>
                  <span className="category-label">{category.label}</span>
                </div>
                <div className="category-items">
                  {category.elements.map(element => (
                    <button
                      key={element}
                      className="element-item"
                      onClick={() => handleSelect(element)}
                      title={ELEMENT_INFO[element]?.description}
                    >
                      <span className="element-name">{element}</span>
                      <span className="element-desc">
                        {ELEMENT_INFO[element]?.description}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            ))}

            {filteredCategories.length === 0 && (
              <div className="no-results">
                No elements match "{searchTerm}"
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
