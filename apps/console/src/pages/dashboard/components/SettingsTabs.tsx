import "./SettingsTabs.css";

interface Props {
  id: string;
  tabs: { id: string; label: string }[];
  selected: string;
  disabled?: boolean;
  orientation?: "horizontal" | "vertical";
  onSelect: (id: string) => void;
}

export function SettingsTabs({
  id,
  tabs,
  selected,
  disabled,
  orientation = "horizontal",
  onSelect,
}: Props) {
  return (
    <div
      className={`settings-tabs${orientation === "vertical" ? " settings-tabs-vertical" : ""}`}
      role="tablist"
      aria-orientation={orientation}
      aria-label="Settings sections"
    >
      {tabs.map((tab, index) => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          id={`${id}-${tab.id}-tab`}
          aria-controls={`${id}-${tab.id}-panel`}
          aria-selected={selected === tab.id}
          tabIndex={selected === tab.id ? 0 : -1}
          disabled={disabled}
          onClick={() => onSelect(tab.id)}
          onKeyDown={(event) => {
            let next: number;
            if (event.key === (orientation === "vertical" ? "ArrowDown" : "ArrowRight")) next = (index + 1) % tabs.length;
            else if (event.key === (orientation === "vertical" ? "ArrowUp" : "ArrowLeft"))
              next = (index - 1 + tabs.length) % tabs.length;
            else if (event.key === "Home") next = 0;
            else if (event.key === "End") next = tabs.length - 1;
            else return;
            event.preventDefault();
            onSelect(tabs[next].id);
            document.getElementById(`${id}-${tabs[next].id}-tab`)?.focus();
          }}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
