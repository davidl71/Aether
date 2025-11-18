import { useEffect, useRef } from 'react';

interface TabDefinition {
  id: string;
  title: string;
}

interface TabNavigationProps {
  tabs: TabDefinition[];
  activeTab: string;
  onSelect: (id: string) => void;
}

export function TabNavigation({ tabs, activeTab, onSelect }: TabNavigationProps) {
  const tabRefs = useRef<Map<string, HTMLButtonElement>>(new Map());
  const activeIndex = tabs.findIndex((tab) => tab.id === activeTab);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Tab key: Navigate between tabs
      if (event.key === 'Tab' && !event.shiftKey && event.target === document.body) {
        event.preventDefault();
        const firstTab = tabs[0];
        if (firstTab) {
          const firstButton = tabRefs.current.get(firstTab.id);
          if (firstButton) {
            firstButton.focus();
          }
        }
        return;
      }

      // Arrow keys: Navigate between tabs
      if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
        const currentIndex = tabs.findIndex((tab) => tab.id === activeTab);
        if (currentIndex === -1) return;

        let nextIndex: number;
        if (event.key === 'ArrowLeft') {
          nextIndex = currentIndex > 0 ? currentIndex - 1 : tabs.length - 1;
        } else {
          nextIndex = currentIndex < tabs.length - 1 ? currentIndex + 1 : 0;
        }

        const nextTab = tabs[nextIndex];
        if (nextTab) {
          event.preventDefault();
          onSelect(nextTab.id);
          // Focus the tab button
          setTimeout(() => {
            const nextButton = tabRefs.current.get(nextTab.id);
            if (nextButton) {
              nextButton.focus();
            }
          }, 0);
        }
      }

      // Number keys 1-5: Jump to specific tab
      const numKey = parseInt(event.key, 10);
      if (numKey >= 1 && numKey <= tabs.length && !event.ctrlKey && !event.metaKey && !event.altKey) {
        const targetTab = tabs[numKey - 1];
        if (targetTab) {
          event.preventDefault();
          onSelect(targetTab.id);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [tabs, activeTab, onSelect]);

  const setTabRef = (tabId: string, element: HTMLButtonElement | null) => {
    if (element) {
      tabRefs.current.set(tabId, element);
    } else {
      tabRefs.current.delete(tabId);
    }
  };

  return (
    <nav className="tab-nav" aria-label="Primary">
      {tabs.map((tab, index) => (
        <button
          key={tab.id}
          ref={(el) => setTabRef(tab.id, el)}
          type="button"
          className={`tab-nav__item ${tab.id === activeTab ? 'tab-nav__item--active' : ''}`}
          onClick={() => onSelect(tab.id)}
          aria-selected={tab.id === activeTab}
          aria-label={`${tab.title} (Press ${index + 1} to jump)`}
          title={`${tab.title} (${index + 1})`}
        >
          {tab.title}
        </button>
      ))}
    </nav>
  );
}
