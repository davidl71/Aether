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
  return (
    <nav className="tab-nav" aria-label="Primary">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          type="button"
          className={`tab-nav__item ${tab.id === activeTab ? 'tab-nav__item--active' : ''}`}
          onClick={() => onSelect(tab.id)}
        >
          {tab.title}
        </button>
      ))}
    </nav>
  );
}
