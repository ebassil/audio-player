## ADDED Requirements

### Requirement: Plugin popup window
The system SHALL provide a resizeable popup window containing the plugin list and plugin configuration UI, opened from a button in the main window's header toolbar.

#### Scenario: Open plugin popup via header button
- **WHEN** user clicks the "Plugins" button in the header toolbar
- **THEN** a new window opens displaying the plugin list with drag-reorder, enable/disable toggles, and a UI button for each plugin that has a GUI
- **AND** if the window is already open, it is brought to focus instead of creating a duplicate

#### Scenario: Plugin popup is resizeable
- **WHEN** user drags the edge or corner of the plugin window
- **THEN** the window resizes accordingly
- **AND** the plugin list and UI iframe adapt to the new dimensions

#### Scenario: Plugin list shows all discovered plugins
- **WHEN** the plugin popup opens
- **THEN** it fetches and displays all plugins from `get_plugins()` and graph nodes from `get_graph_nodes()`
- **AND** each plugin card shows name, type badge, enable checkbox, and UI button (if plugin has GUI)

#### Scenario: Plugin enable/disable in popup
- **WHEN** user toggles the enable checkbox on a plugin card
- **THEN** `enable_plugin` is invoked with the node ID and new state
- **AND** the plugin card reflects the updated state

#### Scenario: Plugin drag-reorder in popup
- **WHEN** user drags a plugin card to a new position in the list
- **THEN** `reorder_plugins` is invoked with the new order
- **AND** the list reflects the new order

#### Scenario: Plugin UI iframe in popup
- **WHEN** user clicks the "UI" button on a plugin card
- **THEN** the popup displays the plugin's UI in an iframe below the plugin list (or in a dedicated area)
- **AND** the iframe receives `postMessage` events for `param_change` and invokes `set_plugin_parameter`

#### Scenario: Close plugin popup
- **WHEN** user closes the plugin popup window
- **THEN** plugin state persists in the backend
- **AND** no data is lost

### Requirement: Header toolbar with plugin button
The main window SHALL include a header toolbar (above or within the player controls area) with a "Plugins" button that opens the plugin popup.

#### Scenario: Header toolbar visible
- **WHEN** the application loads
- **THEN** the header toolbar is displayed with a "Plugins" button
